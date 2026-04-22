#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agents;
mod ai;
mod channel_accounts;
mod channels;
mod commands;
mod cron;
mod db;
mod exec;
mod hotkeys;
mod security;
mod ollama;
mod providers;
mod recorder;
mod skills;
mod streaming;
mod telemetry;
#[path = "tray/mod.rs"]
mod tray;
mod utils;
mod workflow;

use agents::AgentManager;
use channel_accounts::ChannelAccountManager;
use channels::ChannelManager;
use commands::{
    agents as agent_commands, channel, channel_accounts as channel_account_commands, chat,
    cron as cron_commands, developer, gateway as gateway_commands, hotkey as hotkey_commands,
    marketplace as marketplace_commands, ollama as ollama_commands, plugin,
    proxy as proxy_commands, recorder as recorder_commands, settings, wasm,
    workflow as workflow_commands, wizard as wizard_commands, workspace,
};
use cron::CronScheduler;
use db::connection::Database;
use gateway_commands::GatewayManager;
use hotkeys::HotkeyRegistry;
use ollama::manager::OllamaManager;
use recorder::Recorder;
use skills::host::WasmHost;
use std::sync::Arc;
use tauri::Manager;
use telemetry::logging::setup_logging;
use tracing::info;
use utils::config::Config;
use workflow::WorkflowManager;

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
    let hotkey_registry = Arc::new(HotkeyRegistry::new());
    let recorder = Arc::new(Recorder::new(shared_db.clone()));
    let workflow_manager = Arc::new(WorkflowManager::new(shared_db.clone()));
    let marketplace_manager = Arc::new(marketplace_commands::MarketplaceManager::new());

    let cron_scheduler = Arc::new(CronScheduler::new(db.conn()));
    tauri::async_runtime::block_on(async {
        cron_scheduler
            .load_jobs()
            .await
            .expect("Failed to load cron jobs");
        cron_scheduler.start().await;
    });

    tauri::async_runtime::block_on(async {
        hotkey_registry
            .load_enabled_from_db(shared_db.as_ref())
            .await
            .expect("Failed to load hotkeys");
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
            hotkey_commands::get_all_hotkeys,
            hotkey_commands::add_hotkey,
            hotkey_commands::update_hotkey,
            hotkey_commands::delete_hotkey,
            hotkey_commands::register_hotkeys,
            hotkey_commands::get_registered_hotkeys,
            recorder_commands::start_recording,
            recorder_commands::stop_recording,
            recorder_commands::pause_recording,
            recorder_commands::resume_recording,
            recorder_commands::add_event,
            recorder_commands::get_recording_state,
            recorder_commands::get_current_recording,
            recorder_commands::list_recordings,
            recorder_commands::get_recording,
            recorder_commands::delete_recording,
            recorder_commands::playback_recording,
            workflow_commands::create_workflow,
            workflow_commands::get_all_workflows,
            workflow_commands::get_workflow,
            workflow_commands::update_workflow,
            workflow_commands::execute_workflow,
            workflow_commands::delete_workflow,
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
            marketplace_commands::search_marketplace,
            marketplace_commands::get_skill_details,
            marketplace_commands::install_skill,
            marketplace_commands::uninstall_skill,
            marketplace_commands::list_installed_skills,
            marketplace_commands::get_skill_categories,
            plugin::get_all_plugins,
            plugin::get_plugin,
            plugin::install_plugin,
            plugin::enable_plugin,
            plugin::disable_plugin,
            plugin::uninstall_plugin,
            developer::run_diagnostics,
            developer::get_telemetry_data,
            developer::get_token_usage_stats,
            developer::export_telemetry_data,
            developer::clear_telemetry_data,
            developer::get_system_info,
            developer::get_app_logs,
            proxy_commands::get_proxy_config,
            proxy_commands::set_proxy_config,
            proxy_commands::enable_proxy,
            proxy_commands::disable_proxy,
            proxy_commands::test_proxy_connection,
            proxy_commands::get_default_bypass_rules,
            proxy_commands::reset_proxy_config,
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
            wizard_commands::wizard_get_state,
            wizard_commands::wizard_save_state,
            wizard_commands::wizard_complete,
            wizard_commands::wizard_save_api_key,
            wizard_commands::wizard_reset,
        ])
        .setup(move |app| {
            hotkey_registry.set_app_handle(app.handle().clone());
            hotkey_registry.start_listener();

            app.manage(db.clone());
            app.manage(shared_db.clone());
            app.manage(agent_manager.clone());
            app.manage(channel_manager);
            app.manage(channel_account_manager.clone());
            app.manage(gateway_manager.clone());
            app.manage(hotkey_registry.clone());
            app.manage(recorder.clone());
            app.manage(workflow_manager.clone());
            app.manage(marketplace_manager.clone());
            app.manage(cron_scheduler.clone());
            app.manage(wasm_host.clone());
            app.manage(ollama_manager);

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = tray::hide_main_window(&app_handle);
                    }
                    tauri::WindowEvent::Destroyed => {}
                    _ => {
                        let show_enabled = !tray::is_window_visible(&app_handle);
                        let hide_enabled = !show_enabled;
                        let _ = tray::update_tray_menu(&app_handle, show_enabled, hide_enabled);
                    }
                });
            }

            tray::init_tray(&app.handle())?;

            info!("CEOClaw initialized successfully");
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}
