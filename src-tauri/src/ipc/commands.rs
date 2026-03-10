use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{
    agent::{approval::PendingApproval, r#loop::start_message_run},
    db::{
        messages::{create_message, create_message_with_metadata, MessageMetadata, MessageRecord},
        sessions::{
            create_session as create_session_record, list_sessions as list_session_records,
            SessionSummary,
        },
        tools::{
            list_tool_settings as list_tool_setting_records,
            set_tool_enabled as set_tool_enabled_record,
        },
        tool_calls::{
            complete_tool_call as complete_tool_call_record,
            list_pending_tool_calls as list_pending_tool_call_records,
            mark_tool_call_status,
            reject_tool_call as reject_tool_call_record,
        },
    },
    oauth::{clear_oauth_session, start_oauth_login},
    providers::openai::{continue_after_function_output, list_models as list_openai_models},
    settings::{
        load_provider_config, load_provider_status, save_provider_config, set_preferred_auth,
        PreferredAuth, ProviderStatus, SaveProviderConfigInput,
    },
    state::AppState,
    tools::browser::{execute_browser, BrowserRequest},
    tools::filesystem::{execute_filesystem, FilesystemRequest},
    tools::{is_builtin_tool, registry_entries, registry_entry, ToolRegistryEntry},
    tools::shell::execute_shell,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolCallResultPayload {
    call_id: String,
    session_id: String,
    result: String,
}

#[derive(Debug, Clone, Serialize)]
struct AgentStatusPayload<'a> {
    status: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorPayload {
    message: String,
}

#[derive(Debug, Clone)]
struct ToolExecutionOutcome {
    result: String,
    result_message_id: String,
}

#[tauri::command]
pub async fn send_message(app: AppHandle, session_id: String, text: String) -> Result<(), String> {
    start_message_run(app, session_id, text).await
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>, String> {
    list_session_records(&state.db).await.map_err(Into::into)
}

#[tauri::command]
pub async fn create_session(
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<SessionSummary, String> {
    create_session_record(&state.db, title.as_deref().unwrap_or("New chat"))
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_session_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<MessageRecord>, String> {
    crate::db::messages::list_messages(&state.db, &session_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn list_pending_tool_calls(
    state: State<'_, AppState>,
) -> Result<Vec<PendingApproval>, String> {
    list_pending_tool_call_records(&state.db)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn approve_tool_call(
    app: AppHandle,
    call_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pending = {
        let mut approval_state = state.approval.lock().await;
        let position = approval_state
            .pending
            .iter()
            .position(|item| item.id == call_id)
            .ok_or_else(|| "Pending tool call not found".to_string())?;
        approval_state.pending.remove(position)
    };

    mark_tool_call_status(&state.db, &pending.id, "approved")
        .await
        .map_err(|error| error.to_string())?;
    app.emit(
        "agent:status",
        AgentStatusPayload {
            status: "executing_tool",
        },
    )
    .map_err(|error| error.to_string())?;

    let execution = match run_approved_tool(&app, &pending, &state).await {
        Ok(execution) => execution,
        Err(error) => {
            app.emit("agent:status", AgentStatusPayload { status: "idle" })
                .map_err(|emit_error| emit_error.to_string())?;
            app.emit(
                "agent:error",
                ErrorPayload {
                    message: error.clone(),
                },
            )
            .map_err(|emit_error| emit_error.to_string())?;
            return Err(error);
        }
    };

    complete_tool_call_record(
        &state.db,
        &pending.id,
        &execution.result,
        &execution.result_message_id,
    )
    .await
    .map_err(|error| error.to_string())?;
    app.emit(
        "agent:tool-call-result",
        ToolCallResultPayload {
            call_id: pending.id.clone(),
            session_id: pending.session_id.clone(),
            result: execution.result,
        },
    )
    .map_err(|error| error.to_string())?;
    app.emit("agent:status", AgentStatusPayload { status: "idle" })
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reject_tool_call(
    app: AppHandle,
    call_id: String,
    reason: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pending = {
        let mut approval_state = state.approval.lock().await;
        let position = approval_state
            .pending
            .iter()
            .position(|item| item.id == call_id)
            .ok_or_else(|| "Pending tool call not found".to_string())?;
        approval_state.pending.remove(position)
    };

    let rejection_reason = reason.unwrap_or_else(|| "No reason provided.".to_string());
    let message = create_message_with_metadata(
        &state.db,
        &pending.session_id,
        "assistant",
        &rejection_reason,
        MessageMetadata {
            tool_call_id: Some(pending.id.clone()),
            tool_message_kind: Some("rejection".to_string()),
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    reject_tool_call_record(&state.db, &pending.id, &rejection_reason, &message.id)
        .await
        .map_err(|error| error.to_string())?;
    app.emit(
        "agent:tool-call-result",
        ToolCallResultPayload {
            call_id: pending.id,
            session_id: pending.session_id,
            result: rejection_reason,
        },
    )
    .map_err(|error| error.to_string())?;
    app.emit("agent:status", AgentStatusPayload { status: "idle" })
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_provider_status_command(app: AppHandle) -> Result<ProviderStatus, String> {
    load_provider_status(&app)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_tools(state: State<'_, AppState>) -> Result<Vec<ToolRegistryEntry>, String> {
    let enabled_map = list_tool_setting_records(&state.db)
        .await
        .map_err(|error| error.to_string())?;
    Ok(registry_entries(&enabled_map))
}

#[tauri::command]
pub async fn set_tool_enabled_command(
    tool_name: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<ToolRegistryEntry, String> {
    if !is_builtin_tool(&tool_name) {
        return Err(format!("Unknown tool {}", tool_name));
    }

    set_tool_enabled_record(&state.db, &tool_name, enabled)
        .await
        .map_err(|error| error.to_string())?;

    let enabled_map = list_tool_setting_records(&state.db)
        .await
        .map_err(|error| error.to_string())?;

    registry_entry(&tool_name, &enabled_map)
        .ok_or_else(|| format!("Tool {} is missing from the registry", tool_name))
}

#[tauri::command]
pub async fn list_provider_models(app: AppHandle) -> Result<Vec<String>, String> {
    let config = load_provider_config(&app).map_err(|error| error.to_string())?;
    list_openai_models(&config)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn save_provider_config_command(
    app: AppHandle,
    config: SaveProviderConfigInput,
) -> Result<ProviderStatus, String> {
    save_provider_config(&app, config).map_err(|error| error.to_string())?;
    load_provider_status(&app)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_oauth_login_command(app: AppHandle) -> Result<ProviderStatus, String> {
    start_oauth_login()
        .await
        .map_err(|error| error.to_string())?;
    set_preferred_auth(&app, PreferredAuth::Oauth).map_err(|error| error.to_string())?;
    load_provider_status(&app)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn clear_oauth_session_command(app: AppHandle) -> Result<ProviderStatus, String> {
    clear_oauth_session().map_err(|error| error.to_string())?;
    set_preferred_auth(&app, PreferredAuth::Auto).map_err(|error| error.to_string())?;
    load_provider_status(&app)
        .await
        .map_err(|error| error.to_string())
}

async fn run_approved_tool(
    app: &AppHandle,
    pending: &PendingApproval,
    state: &State<'_, AppState>,
) -> Result<ToolExecutionOutcome, String> {
    let payload: serde_json::Value =
        serde_json::from_str(&pending.arguments_json).map_err(|error| error.to_string())?;
    let result = match pending.tool_name.as_str() {
        "shell" => {
            let command = payload
                .get("command")
                .and_then(|value| value.as_str())
                .ok_or_else(|| "Shell command payload is missing command".to_string())?;
            execute_shell(command)
                .await
                .map_err(|error| error.to_string())?
        }
        "filesystem" => {
            let request: FilesystemRequest =
                serde_json::from_value(payload).map_err(|error| error.to_string())?;
            execute_filesystem(&state.workspace_root, request)
                .await
                .map_err(|error| error.to_string())?
        }
        "browser" => {
            let request: BrowserRequest =
                serde_json::from_value(payload).map_err(|error| error.to_string())?;
            execute_browser(request)
                .await
                .map_err(|error| error.to_string())?
        }
        other => return Err(format!("Unsupported tool {}", other)),
    };
    let tool_message = create_message_with_metadata(
        &state.db,
        &pending.session_id,
        "tool",
        &result,
        MessageMetadata {
            tool_call_id: Some(pending.id.clone()),
            tool_message_kind: Some("result".to_string()),
        },
    )
    .await
    .map_err(|error| error.to_string())?;

    let response_id = pending
        .response_id
        .as_deref()
        .ok_or_else(|| "Pending tool call is missing response_id".to_string())?;
    let call_id = pending
        .tool_call_id
        .as_deref()
        .ok_or_else(|| "Pending tool call is missing tool_call_id".to_string())?;
    let config = load_provider_config(app).map_err(|error| error.to_string())?;
    let assistant_text = continue_after_function_output(&config, response_id, call_id, &result)
        .await
        .map_err(|error| error.to_string())?;

    create_message(&state.db, &pending.session_id, "assistant", &assistant_text)
        .await
        .map_err(|error| error.to_string())?;

    app.emit(
        "agent:message-delta",
        serde_json::json!({
            "sessionId": pending.session_id,
            "messageId": uuid::Uuid::new_v4().to_string(),
            "delta": assistant_text
        }),
    )
    .map_err(|error| error.to_string())?;

    Ok(ToolExecutionOutcome {
        result,
        result_message_id: tool_message.id,
    })
}
