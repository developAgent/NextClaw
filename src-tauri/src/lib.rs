#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod agents;
mod channel_accounts;
mod channels;
mod commands;
mod cron;
mod db;
mod exec;
mod hotkeys;
mod ollama;
mod providers;
mod skills;
mod streaming;
mod telemetry;
mod ui;
mod utils;

use agents::AgentManager;
use commands::{agents as agent_commands, chat, settings, ollama as ollama_commands};
use db::connection::Database;
use ollama::manager::OllamaManager;
use telemetry::logging::setup_logging;
use utils::config::Config;
use std::sync::Arc;

use anyhow::Result;
use tauri::Manager;
use tracing::info;

/// Initialize the application
#[allow(clippy::too_many_lines)]
pub fn run() {
    // Setup logging first
    setup_logging();
    info!("CEOClaw starting...");

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Initialize database
    let db = Arc::new(Database::new(&config.data_dir).expect("Failed to initialize database"));

    // Initialize Tauri builder
    tauri::Builder::default()
        // Setup Tauri commands
        .invoke_handler(tauri::generate_handler![
            // Agent commands
            agent_commands::create_agent,
            agent_commands::get_all_agents,
            agent_commands::get_agent,
            agent_commands::update_agent,
            agent_commands::delete_agent,
            agent_commands::clone_agent,
            // Chat commands
            chat::send_message,
            chat::get_chat_history,
            chat::create_session,
            chat::list_sessions,
            chat::delete_session,
            // Settings commands
            settings::get_config,
            settings::update_config,
            settings::set_api_key,
        ])
        .setup(move |app| {
            // Manage database
            app.manage(db.clone());

            // Initialize Agent manager
            app.manage(Arc::new(AgentManager::new(db.conn())));

            info!("CEOClaw initialized successfully");

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}