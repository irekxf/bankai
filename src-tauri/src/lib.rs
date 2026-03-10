mod agent;
mod db;
mod error;
mod ipc;
mod oauth;
mod providers;
mod settings;
mod state;
mod tools;

use tauri::Manager;
use tracing_subscriber::FmtSubscriber;

pub fn run() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db = tauri::async_runtime::block_on(db::init(&app_handle))
                .map_err(|error| error.to_string())?;
            let pending_tool_calls =
                tauri::async_runtime::block_on(db::tool_calls::list_pending_tool_calls(&db))
                    .map_err(|error| error.to_string())?;
            let workspace_root = std::env::current_dir()
                .map_err(|error| error.to_string())?
                .canonicalize()
                .map_err(|error| error.to_string())?;
            app.manage(state::AppState::new(
                db,
                workspace_root,
                agent::approval::ApprovalState::new(pending_tool_calls),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::send_message,
            ipc::commands::list_sessions,
            ipc::commands::create_session,
            ipc::commands::get_session_messages,
            ipc::commands::list_pending_tool_calls,
            ipc::commands::approve_tool_call,
            ipc::commands::reject_tool_call,
            ipc::commands::get_oauth_status_command,
            ipc::commands::start_oauth_login_command,
            ipc::commands::clear_oauth_session_command,
            ipc::commands::get_provider_config,
            ipc::commands::list_provider_models,
            ipc::commands::save_provider_config_command,
            ipc::commands::list_tools,
            ipc::commands::set_tool_enabled_command
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Bankai");
}
