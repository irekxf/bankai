use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use crate::{
    agent::approval::PendingApproval,
    db::{messages::create_message, sessions::{ensure_session, touch_session}},
    providers::openai::stream_chat_response,
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

    if let Some(command) = text.strip_prefix("/shell ").map(str::trim).filter(|value| !value.is_empty()) {
        let approval = PendingApproval {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            tool_name: "shell".to_string(),
            arguments_json: format!(r#"{{"command":"{}"}}"#, command.replace('"', "\\\"")),
        };

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
        touch_session(&db, &session_id, Some("/shell request"))
            .await
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    let message_id = Uuid::new_v4().to_string();
    let mut full_response = String::new();

    match stream_chat_response(&config, &text, |delta| {
        full_response.push_str(&delta);
        app.emit(
            "agent:message-delta",
            MessageDeltaPayload {
                session_id: session_id.clone(),
                message_id: message_id.clone(),
                delta,
            },
        )
        .map_err(|error| crate::error::AppError::Message(error.to_string()))
    })
    .await
    {
        Ok(()) => {
            create_message(&db, &session_id, "assistant", &full_response)
                .await
                .map_err(|error| error.to_string())?;
            let title = text.lines().next().map(|line| line.chars().take(60).collect::<String>());
            touch_session(&db, &session_id, title.as_deref())
                .await
                .map_err(|error| error.to_string())?;
            app.emit("agent:status", AgentStatusPayload { status: "idle" })
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
