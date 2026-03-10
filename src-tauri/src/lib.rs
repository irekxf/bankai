mod agent;
mod db;
mod error;
mod ipc;
mod providers;
mod settings;
mod state;
mod tools;

use tauri::Manager;
use tracing_subscriber::FmtSubscriber;

pub fn run() {
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db = tauri::async_runtime::block_on(db::init(&app_handle))
                .map_err(|error| error.to_string())?;
            app.manage(state::AppState::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::send_message,
            ipc::commands::list_sessions,
            ipc::commands::create_session,
            ipc::commands::get_session_messages,
            ipc::commands::approve_tool_call,
            ipc::commands::reject_tool_call,
            ipc::commands::get_provider_config,
            ipc::commands::save_provider_config_command
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Bankai");
}
