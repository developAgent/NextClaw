use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tracing::{error, info};

const TRAY_ID: &str = "main";
const MENU_SHOW: &str = "tray_show";
const MENU_HIDE: &str = "tray_hide";
const MENU_QUIT: &str = "tray_quit";

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<tauri::WebviewWindow<R>> {
    app.get_webview_window("main")
}

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Window", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, MENU_HIDE, "Hide Window", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &hide, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("CEOClaw")
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_icon_event);

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    let visible = is_window_visible(app);
    update_tray_menu(app, !visible, visible)?;
    info!("System tray initialized");
    Ok(())
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    if let Err(error) = process_menu_event(app, event.id().as_ref()) {
        error!("Failed to handle tray menu event: {}", error);
    }
}

fn process_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) -> tauri::Result<()> {
    match id {
        MENU_SHOW => show_main_window(app)?,
        MENU_HIDE => hide_main_window(app)?,
        MENU_QUIT => {
            info!("Quit clicked from system tray");
            app.exit(0);
        }
        _ => {}
    }

    Ok(())
}

fn handle_tray_icon_event<R: Runtime>(tray: &tauri::tray::TrayIcon<R>, event: TrayIconEvent) {
    let app = tray.app_handle();

    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            if let Err(error) = toggle_main_window(app) {
                error!("Failed to toggle main window from tray: {}", error);
            }
        }
        _ => {}
    }
}

pub fn init_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    create_tray(app)
}

pub fn update_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    show_enabled: bool,
    hide_enabled: bool,
) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Window", show_enabled, None::<&str>)?;
    let hide = MenuItem::with_id(app, MENU_HIDE, "Hide Window", hide_enabled, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &separator, &quit])?;

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}

pub fn show_tray_notification<R: Runtime>(app: &AppHandle<R>, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        error!("Failed to show tray notification: {}", error);
    }
}

pub fn is_window_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    main_window(app)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = main_window(app) {
        window.show()?;
        window.set_focus()?;
        update_tray_menu(app, false, true)?;
    }

    Ok(())
}

pub fn hide_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = main_window(app) {
        window.hide()?;
        update_tray_menu(app, true, false)?;
    }

    Ok(())
}

pub fn toggle_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if is_window_visible(app) {
        hide_main_window(app)
    } else {
        show_main_window(app)
    }
}
