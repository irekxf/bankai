use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use crate::{
    agent::approval::PendingApproval,
    db::{
        messages::create_message,
        sessions::{ensure_session, touch_session},
        tool_calls::create_pending_tool_call,
    },
    providers::openai::{create_tool_aware_response, ModelTurn},
    settings::load_provider_config,
    state::AppState,
};

#[derive(Debug, Clone, Serialize)]
struct AgentStatusPayload<'a> {
    status: &'a str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MessageDeltaPayload {
    session_id: String,
    message_id: String,
    delta: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorPayload {
    message: String,
}

pub async fn start_message_run(
    app: AppHandle,
    session_id: String,
    text: String,
) -> Result<(), String> {
    app.emit("agent:status", AgentStatusPayload { status: "thinking" })
        .map_err(|error| error.to_string())?;

    let config = load_provider_config(&app).map_err(|error| error.to_string())?;
    let db = {
        let state = app.state::<AppState>();
        state.db.clone()
    };

    ensure_session(&db, &session_id, "New chat")
        .await
        .map_err(|error| error.to_string())?;
    create_message(&db, &session_id, "user", &text)
        .await
        .map_err(|error| error.to_string())?;

    match create_tool_aware_response(&config, &text).await {
        Ok(ModelTurn::Text(full_response)) => {
            let message_id = Uuid::new_v4().to_string();
            app.emit(
                "agent:message-delta",
                MessageDeltaPayload {
                    session_id: session_id.clone(),
                    message_id,
                    delta: full_response.clone(),
                },
            )
            .map_err(|error| error.to_string())?;
            create_message(&db, &session_id, "assistant", &full_response)
                .await
                .map_err(|error| error.to_string())?;
            let title = text
                .lines()
                .next()
                .map(|line| line.chars().take(60).collect::<String>());
            touch_session(&db, &session_id, title.as_deref())
                .await
                .map_err(|error| error.to_string())?;
            app.emit("agent:status", AgentStatusPayload { status: "idle" })
                .map_err(|error| error.to_string())?;
            Ok(())
        }
        Ok(ModelTurn::ToolCall(call)) => {
            let approval = PendingApproval {
                id: Uuid::new_v4().to_string(),
                session_id: session_id.clone(),
                response_id: Some(call.response_id),
                tool_call_id: Some(call.call_id),
                tool_name: call.name,
                arguments_json: call.arguments,
            };

            create_pending_tool_call(&db, &approval)
                .await
                .map_err(|error| error.to_string())?;

            {
                let state = app.state::<AppState>();
                let mut approval_state = state.approval.lock().await;
                approval_state.pending.push(approval.clone());
            }

            app.emit("agent:tool-call-request", &approval)
                .map_err(|error| error.to_string())?;
            app.emit(
                "agent:status",
                AgentStatusPayload {
                    status: "awaiting_approval",
                },
            )
            .map_err(|error| error.to_string())?;
            touch_session(&db, &session_id, Some("tool request"))
                .await
                .map_err(|error| error.to_string())?;
            Ok(())
        }
        Err(error) => {
            app.emit(
                "agent:error",
                ErrorPayload {
                    message: error.to_string(),
                },
            )
            .map_err(|emit_error| emit_error.to_string())?;
            app.emit("agent:status", AgentStatusPayload { status: "idle" })
                .map_err(|emit_error| emit_error.to_string())?;
            Err(error.to_string())
        }
    }
}
