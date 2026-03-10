mod agent;
mod error;
mod ipc;
mod state;

use tauri::Manager;
use tracing_subscriber::FmtSubscriber;

pub fn run() {
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    tauri::Builder::default()
        .manage(state::AppState::default())
        .setup(|app| {
            let _app_handle = app.app_handle();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::send_message,
            ipc::commands::list_sessions,
            ipc::commands::approve_tool_call,
            ipc::commands::reject_tool_call
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Bankai");
}
