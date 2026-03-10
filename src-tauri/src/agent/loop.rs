use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use crate::{agent::approval::PendingApproval, state::AppState};

#[derive(Debug, Clone, Serialize)]
struct AgentStatusPayload<'a> {
    status: &'a str,
}

pub async fn start_message_run(app: AppHandle, session_id: String, text: String) -> Result<(), String> {
    app.emit("agent:status", AgentStatusPayload { status: "thinking" })
        .map_err(|error| error.to_string())?;

    let approval = PendingApproval {
        id: Uuid::new_v4().to_string(),
        tool_name: "shell".to_string(),
        arguments_json: format!(r#"{{"command":"echo {}"}}"#, text),
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

    let _ = session_id;
    Ok(())
}
