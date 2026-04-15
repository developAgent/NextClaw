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
use channel_accounts::ChannelAccountManager;
use channels::ChannelManager;
use commands::{
    agents as agent_commands, channel, channel_accounts as channel_account_commands, chat, cron as cron_commands,
    developer, gateway as gateway_commands, ollama as ollama_commands, settings, wasm, workspace,
};
use cron::CronScheduler;
use db::connection::Database;
use ollama::manager::OllamaManager;
use skills::host::WasmHost;
use std::sync::Arc;
use telemetry::logging::setup_logging;
use tracing::info;
use utils::config::Config;
use gateway_commands::GatewayManager;

/// Initialize the application
#[allow(clippy::too_many_lines)]
pub fn run() {
    setup_logging();
    info!("CEOClaw starting...");

    let config = Config::load().expect("Failed to load configuration");
    let db = Database::new(&config.data_dir).expect("Failed to initialize database");
    let shared_db = Arc::new(db.clone());

    let agent_manager = Arc::new(AgentManager::new(db.conn()));
    let channel_manager = ChannelManager::new(shared_db.clone());
    channel_manager
        .initialize_blocking()
        .expect("Failed to initialize channel manager");

    let channel_account_manager = Arc::new(ChannelAccountManager::new(db.conn()));
    let gateway_manager = Arc::new(GatewayManager::new());

    let cron_scheduler = Arc::new(CronScheduler::new(db.conn()));
    tauri::async_runtime::block_on(async {
        cron_scheduler
            .load_jobs()
            .await
            .expect("Failed to load cron jobs");
        cron_scheduler.start().await;
    });

    let wasm_host = Arc::new(WasmHost::default(shared_db.clone()));
    tauri::async_runtime::block_on(async {
        wasm_host
            .initialize()
            .await
            .expect("Failed to initialize WASM host");
    });

    let mut ollama_manager = OllamaManager::new(shared_db.clone());
    tauri::async_runtime::block_on(async {
        ollama_manager
            .initialize()
            .await
            .expect("Failed to initialize Ollama manager");
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            agent_commands::create_agent,
            agent_commands::get_all_agents,
            agent_commands::get_agent,
            agent_commands::update_agent,
            agent_commands::delete_agent,
            agent_commands::clone_agent,
            chat::send_message,
            chat::get_chat_history,
            chat::create_session,
            chat::list_sessions,
            chat::delete_session,
            settings::get_config,
            settings::update_config,
            settings::set_api_key,
            settings::delete_api_key,
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
            channel_account_commands::create_channel_account,
            channel_account_commands::get_all_channel_accounts,
            channel_account_commands::get_channel_accounts,
            channel_account_commands::get_channel_account,
            channel_account_commands::update_channel_account,
            channel_account_commands::delete_channel_account,
            channel_account_commands::set_default_channel_account,
            cron_commands::create_cron_job,
            cron_commands::get_all_cron_jobs,
            cron_commands::get_cron_job,
            cron_commands::update_cron_job,
            cron_commands::delete_cron_job,
            cron_commands::get_cron_executions,
            cron_commands::start_cron_scheduler,
            cron_commands::stop_cron_scheduler,
            cron_commands::run_cron_job,
            wasm::wasm_host_initialized,
            wasm::wasm_list_skills,
            wasm::wasm_get_skill_manifest,
            wasm::wasm_execute_skill,
            wasm::wasm_register_skill,
            wasm::wasm_unregister_skill,
            wasm::wasm_set_skill_enabled,
            wasm::wasm_is_skill_registered,
            gateway_commands::get_gateway_status,
            gateway_commands::start_gateway,
            gateway_commands::stop_gateway,
            gateway_commands::restart_gateway,
            gateway_commands::check_gateway_health,
            gateway_commands::get_gateway_config,
            gateway_commands::update_gateway_config,
            gateway_commands::generate_new_token,
            gateway_commands::get_control_ui_url,
            developer::get_app_logs,
            workspace::list_workspaces,
            workspace::get_current_workspace,
            workspace::create_workspace,
            workspace::set_current_workspace,
            ollama_commands::ollama_check_connection,
            ollama_commands::ollama_list_models,
            ollama_commands::ollama_refresh_models,
            ollama_commands::ollama_get_model,
            ollama_commands::ollama_pull_model,
            ollama_commands::ollama_delete_model,
            ollama_commands::ollama_chat,
            ollama_commands::ollama_generate,
            ollama_commands::ollama_embed,
        ])
        .setup(move |app| {
            app.manage(db.clone());
            app.manage(shared_db.clone());
            app.manage(agent_manager.clone());
            app.manage(channel_manager);
            app.manage(channel_account_manager.clone());
            app.manage(gateway_manager.clone());
            app.manage(cron_scheduler.clone());
            app.manage(wasm_host.clone());
            app.manage(ollama_manager);

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
