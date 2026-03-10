use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{agent::r#loop::start_message_run, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
}

#[tauri::command]
pub async fn send_message(app: AppHandle, session_id: String, text: String) -> Result<(), String> {
    start_message_run(app, session_id, text).await
}

#[tauri::command]
pub async fn list_sessions() -> Result<Vec<SessionSummary>, String> {
    Ok(vec![SessionSummary {
        id: "local-draft".to_string(),
        title: "ChatGPT agent bootstrap".to_string(),
    }])
}

#[tauri::command]
pub async fn approve_tool_call(call_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut approval_state = state.approval.lock().await;
    approval_state.pending.retain(|item| item.id != call_id);
    Ok(())
}

#[tauri::command]
pub async fn reject_tool_call(
    call_id: String,
    reason: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut approval_state = state.approval.lock().await;
    approval_state.pending.retain(|item| item.id != call_id);
    let _ = reason;
    Ok(())
}
