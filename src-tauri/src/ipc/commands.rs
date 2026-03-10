use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{
    agent::{approval::PendingApproval, r#loop::start_message_run},
    db::{
        messages::{create_message, MessageRecord},
        sessions::{create_session as create_session_record, list_sessions as list_session_records, SessionSummary},
    },
    settings::{load_provider_config, save_provider_config, ProviderConfig, SaveProviderConfigInput},
    state::AppState,
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

    let result = run_approved_tool(&pending, &state).await?;
    app.emit(
        "agent:tool-call-result",
        ToolCallResultPayload {
            call_id: pending.id.clone(),
            session_id: pending.session_id.clone(),
            result,
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

    let message = format!(
        "Tool call rejected: {}",
        reason.unwrap_or_else(|| "no reason provided".to_string())
    );
    create_message(
        &state.db,
        &pending.session_id,
        "assistant",
        &message,
    )
    .await
    .map_err(|error| error.to_string())?;
    app.emit(
        "agent:tool-call-result",
        ToolCallResultPayload {
            call_id: pending.id,
            session_id: pending.session_id,
            result: message,
        },
    )
    .map_err(|error| error.to_string())?;
    app.emit("agent:status", AgentStatusPayload { status: "idle" })
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_provider_config(app: AppHandle) -> Result<ProviderConfig, String> {
    load_provider_config(&app).map_err(Into::into)
}

#[tauri::command]
pub async fn save_provider_config_command(
    app: AppHandle,
    config: SaveProviderConfigInput,
) -> Result<ProviderConfig, String> {
    save_provider_config(&app, config).map_err(Into::into)
}

async fn run_approved_tool(
    pending: &PendingApproval,
    state: &State<'_, AppState>,
) -> Result<String, String> {
    if pending.tool_name != "shell" {
        return Err(format!("Unsupported tool {}", pending.tool_name));
    }

    let payload: serde_json::Value =
        serde_json::from_str(&pending.arguments_json).map_err(|error| error.to_string())?;
    let command = payload
        .get("command")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Shell command payload is missing command".to_string())?;

    let result = execute_shell(command).await.map_err(|error| error.to_string())?;
    create_message(
        &state.db,
        &pending.session_id,
        "tool",
        &format!("shell\n{}", result),
    )
    .await
    .map_err(|error| error.to_string())?;

    Ok(result)
}
