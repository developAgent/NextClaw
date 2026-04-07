#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod channels;
mod commands;
mod db;
mod exec;
mod hotkeys;
mod telemetry;
mod tray;
mod ui;
mod utils;

use channels::manager::ChannelManager;
use commands::{chat, channel, plugin, hotkey, settings};
use db::connection::Database;
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

    // Initialize channel manager
    let channel_manager = Arc::new(ChannelManager::new(db.clone()));

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
            // Channel commands
            channel::get_all_channels,
            channel::get_channel,
            channel::add_channel,
            channel::update_channel,
            channel::delete_channel,
            channel::set_default_channel,
            channel::get_default_channel,
            channel::check_channel_health,
            channel::get_channel_config,
            channel::update_channel_config,
            // Plugin commands
            plugin::get_all_plugins,
            plugin::get_plugin,
            plugin::install_plugin,
            plugin::enable_plugin,
            plugin::disable_plugin,
            plugin::uninstall_plugin,
            // Hotkey commands
            hotkey::get_all_hotkeys,
            hotkey::add_hotkey,
            hotkey::update_hotkey,
            hotkey::delete_hotkey,
            hotkey::register_hotkeys,
            // Settings commands
            settings::get_config,
            settings::update_config,
            settings::set_api_key,
        ])
        .setup(move |app| {
            // Manage database
            app.manage(db.clone());

            // Manage channel manager
            app.manage(channel_manager.clone());

            // Initialize channel manager (blocking call during setup)
            if let Err(e) = channel_manager.initialize_blocking() {
                tracing::error!("Failed to initialize channel manager: {}", e);
            }

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