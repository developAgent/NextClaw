use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tracing::{info, error};

/// Create the system tray with menu items
pub fn create_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show Window");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
    let separator = SystemTrayMenuItem::Separator;

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

/// Handle system tray events
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            info!("System tray left clicked");
            // Toggle window visibility on left click
            let window = app.get_webview_window("main").unwrap();
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SystemTrayEvent::RightClick { .. } => {
            info!("System tray right clicked");
        }
        SystemTrayEvent::DoubleClick { .. } => {
            info!("System tray double clicked");
            let window = app.get_webview_window("main").unwrap();
            let _ = window.show();
            let _ = window.set_focus();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "show" => {
                    info!("Show window clicked");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    info!("Hide window clicked");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "quit" => {
                    info!("Quit clicked");
                    app.exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Initialize the system tray
pub fn init_tray(app: &AppHandle) {
    info!("Initializing system tray");
    // The tray is created in the Builder, so we just need to set up the event handler here
    // The event handler will be registered in the Builder's on_window_event or via the plugin
}

/// Update tray menu item states
pub fn update_tray_menu(app: &AppHandle, show_enabled: bool, hide_enabled: bool) {
    if let Some(tray) = app.tray_handle() {
        let quit = CustomMenuItem::new("quit".to_string(), "Quit");
        let show = CustomMenuItem::new("show".to_string(), "Show Window");
        let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
        let separator = SystemTrayMenuItem::Separator;

        let mut menu = SystemTrayMenu::new();

        if show_enabled {
            menu = menu.add_item(show);
        }
        if hide_enabled {
            menu = menu.add_item(hide);
        }

        menu = menu.add_native_item(separator).add_item(quit);

        let _ = tray.set_menu(menu);
    }
}

/// Show a notification from the tray
pub fn show_tray_notification(app: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app.notification()
        .builder()
        .title(title)
        .body(body)
        .show() {
        error!("Failed to show tray notification: {}", e);
    }
}

/// Get current window visibility state
pub fn is_window_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}