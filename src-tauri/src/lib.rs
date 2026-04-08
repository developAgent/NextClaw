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
mod tray;
mod ui;
mod utils;

use agents::AgentManager;
use channel_accounts::ChannelAccountManager;
use channels::manager::ChannelManager;
use commands::{agents as agent_commands, channel_accounts as channel_account_commands, channel, chat, cron as cron_commands, plugin, hotkey, settings, ollama as ollama_commands, wasm, chat_v2, anthropic};
use cron::CronScheduler;
use db::connection::Database;
use ollama::manager::OllamaManager;
use skills::host::WasmHost;
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
            // Agent commands
            agent_commands::create_agent,
            agent_commands::get_all_agents,
            agent_commands::get_agent,
            agent_commands::update_agent,
            agent_commands::delete_agent,
            agent_commands::clone_agent,
            // Channel account commands
            channel_account_commands::create_channel_account,
            channel_account_commands::get_all_channel_accounts,
            channel_account_commands::get_channel_accounts,
            channel_account_commands::get_channel_account,
            channel_account_commands::update_channel_account,
            channel_account_commands::delete_channel_account,
            channel_account_commands::set_default_channel_account,
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
            // Cron commands
            cron_commands::create_cron_job,
            cron_commands::get_all_cron_jobs,
            cron_commands::get_cron_job,
            cron_commands::update_cron_job,
            cron_commands::delete_cron_job,
            cron_commands::get_cron_executions,
            cron_commands::start_cron_scheduler,
            cron_commands::stop_cron_scheduler,
            cron_commands::run_cron_job,
            // Chat commands
            chat::send_message,
            chat::get_chat_history,
            chat::create_session,
            chat::list_sessions,
            chat::delete_session,
            // Chat V2 commands (OpenAI integration)
            chat_v2::create_chat_completion,
            chat_v2::list_models,
            chat_v2::validate_api_key,
            chat_v2::configure_openai,
            chat_v2::get_openai_status,
            // Anthropic commands
            anthropic::create_anthropic_message,
            anthropic::list_anthropic_models,
            anthropic::validate_anthropic_api_key,
            anthropic::configure_anthropic,
            anthropic::get_anthropic_status,
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
            // Ollama commands
            ollama_commands::ollama_check_connection,
            ollama_commands::ollama_list_models,
            ollama_commands::ollama_refresh_models,
            ollama_commands::ollama_get_model,
            ollama_commands::ollama_pull_model,
            ollama_commands::ollama_delete_model,
            ollama_commands::ollama_chat,
            ollama_commands::ollama_generate,
            ollama_commands::ollama_embed,
            // WASM commands
            wasm::wasm_host_initialized,
            wasm::wasm_list_skills,
            wasm::wasm_get_skill_manifest,
            wasm::wasm_execute_skill,
            wasm::wasm_register_skill,
            wasm::wasm_unregister_skill,
            wasm::wasm_is_skill_registered,
        ])
        .setup(move |app| {
            // Manage database
            app.manage(db.clone());

            // Manage channel manager
            app.manage(channel_manager.clone());

            // Initialize Agent manager
            app.manage(Arc::new(AgentManager::new(db.clone())));

            // Initialize Channel Account manager
            app.manage(Arc::new(ChannelAccountManager::new(db.clone())));

            // Initialize Cron scheduler
            let cron_scheduler = Arc::new(CronScheduler::new(db.clone()));
            app.manage(cron_scheduler.clone());
            info!("Cron scheduler initialized");

            // Initialize OpenAI state
            use commands::chat_v2::OpenAIState;
            app.manage(OpenAIState {
                provider: Arc::new(tokio::sync::Mutex::new(None)),
            });

            // Initialize Anthropic state
            use commands::anthropic::AnthropicState;
            app.manage(AnthropicState {
                provider: Arc::new(tokio::sync::Mutex::new(None)),
            });

            // Initialize Ollama manager
            let mut ollama_manager = OllamaManager::new(db.clone());
            let ollama_manager = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { ollama_manager.initialize().await })
            });
            match ollama_manager {
                Ok(()) => {
                    app.manage(Arc::new(OllamaManager::new(db.clone())));
                    info!("Ollama manager initialized");
                }
                Err(e) => {
                    tracing::warn!("Ollama not available: {}", e);
                    // Still manage the manager even if not connected
                    app.manage(Arc::new(OllamaManager::new(db.clone())));
                }
            }

            // Initialize WASM host
            let wasm_host = Arc::new(WasmHost::default(db.clone()));
            let wasm_host_result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { wasm_host.initialize().await })
            });
            match wasm_host_result {
                Ok(()) => {
                    app.manage(Arc::new(WasmHost::default(db.clone())));
                    info!("WASM host initialized");
                }
                Err(e) => {
                    tracing::warn!("WASM host initialization failed: {}", e);
                    // Still manage the host even if initialization failed
                    app.manage(Arc::new(WasmHost::default(db.clone())));
                }
            }

            // Initialize channel manager (blocking call during setup)
            if let Err(e) = channel_manager.initialize_blocking() {
                tracing::error!("Failed to initialize channel manager: {}", e);
            }

            // Load cron jobs from database
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async {
                        if let Err(e) = cron_scheduler.load_jobs().await {
                            tracing::warn!("Failed to load cron jobs: {}", e);
                        }
                    })
            });

            // Start cron scheduler
            tokio::task::spawn(async move {
                cron_scheduler.start().await;
            });

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