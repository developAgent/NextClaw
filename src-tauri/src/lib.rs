mod ai;
mod commands;
mod db;
mod exec;
mod telemetry;
mod utils;

use commands::{chat, command_exec, file_ops, settings};
use db::connection::Database;
use telemetry::logging::setup_logging;
use utils::config::Config;

use anyhow::Result;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::info;

/// Initialize the application
#[allow(clippy::too_many_lines)]
pub fn run() {
    // Setup logging first
    setup_logging();
    info!("CEOClaw starting...");

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Initialize Tauri builder
    tauri::Builder::default()
        // Setup Tauri commands
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            chat::send_message,
            chat::get_chat_history,
            chat::create_session,
            chat::list_sessions,
            chat::delete_session,
            // Command execution
            command_exec::execute_command,
            command_exec::get_command_history,
            // File operations
            file_ops::list_files,
            file_ops::read_file,
            file_ops::write_file,
            file_ops::get_file_metadata,
            // Settings
            settings::get_config,
            settings::update_config,
            settings::set_api_key,
        ])
        .setup(|app| {
            // Initialize database
            let db = Database::new(&config.data_dir)?;
            app.manage(db);

            // Initialize AI client (lazy, on first use)
            info!("CEOClaw initialized successfully");

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}