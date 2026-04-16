use crate::db::Database;
use crate::utils::error::{AppError, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize)]
pub struct RegisteredHotkey {
    pub id: String,
    pub action: String,
    pub key_combination: String,
}

#[derive(Debug, Clone, Serialize)]
struct HotkeyTriggerEvent {
    hotkey_id: String,
    action: String,
    key_combination: String,
}

#[derive(Debug, Default)]
pub struct HotkeyRegistry {
    registered: Arc<Mutex<HashMap<String, RegisteredHotkey>>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    listener_started: AtomicBool,
}

impl HotkeyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_app_handle(&self, app_handle: AppHandle) {
        let mut guard = self
            .app_handle
            .lock()
            .expect("hotkey app handle lock poisoned");
        *guard = Some(app_handle);
    }

    pub fn start_listener(&self) {
        if self.listener_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let registered = Arc::clone(&self.registered);
        let app_handle = Arc::clone(&self.app_handle);

        std::thread::spawn(move || {
            let pressed_keys = Arc::new(Mutex::new(HashSet::<String>::new()));
            let triggered_hotkeys = Arc::new(Mutex::new(HashSet::<String>::new()));

            let callback_pressed = Arc::clone(&pressed_keys);
            let callback_triggered = Arc::clone(&triggered_hotkeys);
            let callback_registered = Arc::clone(&registered);
            let callback_app_handle = Arc::clone(&app_handle);

            if let Err(error) = rdev::listen(move |event| {
                handle_rdev_event(
                    event,
                    &callback_registered,
                    &callback_app_handle,
                    &callback_pressed,
                    &callback_triggered,
                );
            }) {
                error!("Hotkey listener stopped: {}", error);
            }
        });

        info!("Hotkey listener started");
    }

    pub async fn load_enabled_from_db(&self, db: &Database) -> Result<Vec<RegisteredHotkey>> {
        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        let mut stmt = conn_guard
            .prepare(
                "SELECT id, action, key_combination FROM hotkeys WHERE enabled = 1 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let registered = stmt
            .query_map([], |row| {
                Ok(RegisteredHotkey {
                    id: row.get(0)?,
                    action: row.get(1)?,
                    key_combination: row.get(2)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        drop(stmt);
        drop(conn_guard);

        self.replace_all(registered.clone()).await;
        Ok(registered)
    }

    pub async fn replace_all(&self, hotkeys: Vec<RegisteredHotkey>) {
        let mut guard = self
            .registered
            .lock()
            .expect("hotkey registry lock poisoned");
        guard.clear();

        for hotkey in hotkeys {
            guard.insert(hotkey.id.clone(), hotkey);
        }
    }

    pub async fn remove(&self, id: &str) {
        let mut guard = self
            .registered
            .lock()
            .expect("hotkey registry lock poisoned");
        guard.remove(id);
    }

    pub async fn list(&self) -> Vec<RegisteredHotkey> {
        let guard = self
            .registered
            .lock()
            .expect("hotkey registry lock poisoned");
        guard.values().cloned().collect()
    }
}

fn handle_rdev_event(
    event: rdev::Event,
    registered: &Arc<Mutex<HashMap<String, RegisteredHotkey>>>,
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
    pressed_keys: &Arc<Mutex<HashSet<String>>>,
    triggered_hotkeys: &Arc<Mutex<HashSet<String>>>,
) {
    match event.event_type {
        rdev::EventType::KeyPress(key) => {
            let Some(normalized_key) = normalize_rdev_key(key) else {
                return;
            };

            let pressed_snapshot = {
                let mut pressed = pressed_keys.lock().expect("pressed keys lock poisoned");
                pressed.insert(normalized_key);
                pressed.clone()
            };

            let registered_snapshot = {
                let guard = registered.lock().expect("hotkey registry lock poisoned");
                guard.values().cloned().collect::<Vec<_>>()
            };

            for hotkey in registered_snapshot {
                let expected_keys = parse_key_combination(&hotkey.key_combination);
                if expected_keys.is_empty() || !expected_keys.is_subset(&pressed_snapshot) {
                    continue;
                }

                let mut triggered = triggered_hotkeys
                    .lock()
                    .expect("triggered hotkeys lock poisoned");
                if !triggered.insert(hotkey.id.clone()) {
                    continue;
                }
                drop(triggered);

                if let Err(error) = dispatch_hotkey_action(app_handle, &hotkey) {
                    warn!(
                        "Failed to execute hotkey action {} for {}: {}",
                        hotkey.action, hotkey.key_combination, error
                    );
                }
            }
        }
        rdev::EventType::KeyRelease(key) => {
            let Some(normalized_key) = normalize_rdev_key(key) else {
                return;
            };

            let pressed_snapshot = {
                let mut pressed = pressed_keys.lock().expect("pressed keys lock poisoned");
                pressed.remove(&normalized_key);
                pressed.clone()
            };

            let registered_snapshot = {
                let guard = registered.lock().expect("hotkey registry lock poisoned");
                guard.values().cloned().collect::<Vec<_>>()
            };

            let mut triggered = triggered_hotkeys
                .lock()
                .expect("triggered hotkeys lock poisoned");
            triggered.retain(|hotkey_id| {
                registered_snapshot
                    .iter()
                    .find(|hotkey| hotkey.id == *hotkey_id)
                    .map(|hotkey| {
                        let expected_keys = parse_key_combination(&hotkey.key_combination);
                        expected_keys.is_subset(&pressed_snapshot)
                    })
                    .unwrap_or(false)
            });
        }
        _ => {}
    }
}

fn dispatch_hotkey_action(
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
    hotkey: &RegisteredHotkey,
) -> Result<()> {
    let app = app_handle
        .lock()
        .expect("hotkey app handle lock poisoned")
        .clone()
        .ok_or_else(|| {
            AppError::Internal("Hotkey runtime app handle is not available".to_string())
        })?;

    let action = hotkey.action.trim().to_ascii_lowercase();
    match action.as_str() {
        "toggle_main_window" | "toggle-window" | "toggle window" => {
            crate::tray::toggle_main_window(&app)
                .map_err(|error| AppError::Execution(error.to_string()))?;
        }
        "show_main_window" | "show-window" | "show window" => {
            crate::tray::show_main_window(&app)
                .map_err(|error| AppError::Execution(error.to_string()))?;
        }
        "hide_main_window" | "hide-window" | "hide window" => {
            crate::tray::hide_main_window(&app)
                .map_err(|error| AppError::Execution(error.to_string()))?;
        }
        other => {
            return Err(AppError::Validation(format!(
                "Unsupported hotkey action: {other}. Supported actions: toggle_main_window, show_main_window, hide_main_window"
            )));
        }
    }

    let payload = HotkeyTriggerEvent {
        hotkey_id: hotkey.id.clone(),
        action: hotkey.action.clone(),
        key_combination: hotkey.key_combination.clone(),
    };
    let _ = app.emit("hotkey-triggered", payload);

    info!(
        "Triggered hotkey {} -> {}",
        hotkey.key_combination, hotkey.action
    );
    Ok(())
}

fn parse_key_combination(value: &str) -> HashSet<String> {
    value
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_uppercase())
        .collect()
}

fn normalize_rdev_key(key: rdev::Key) -> Option<String> {
    let normalized = match key {
        rdev::Key::ControlLeft | rdev::Key::ControlRight => "CTRL".to_string(),
        rdev::Key::ShiftLeft | rdev::Key::ShiftRight => "SHIFT".to_string(),
        rdev::Key::Alt | rdev::Key::AltGr => "ALT".to_string(),
        rdev::Key::MetaLeft | rdev::Key::MetaRight => "META".to_string(),
        rdev::Key::Return => "ENTER".to_string(),
        rdev::Key::Tab => "TAB".to_string(),
        rdev::Key::Escape => "ESC".to_string(),
        rdev::Key::Space => "SPACE".to_string(),
        rdev::Key::UpArrow => "UP".to_string(),
        rdev::Key::DownArrow => "DOWN".to_string(),
        rdev::Key::LeftArrow => "LEFT".to_string(),
        rdev::Key::RightArrow => "RIGHT".to_string(),
        rdev::Key::Backspace => "BACKSPACE".to_string(),
        rdev::Key::Num0 => "0".to_string(),
        rdev::Key::Num1 => "1".to_string(),
        rdev::Key::Num2 => "2".to_string(),
        rdev::Key::Num3 => "3".to_string(),
        rdev::Key::Num4 => "4".to_string(),
        rdev::Key::Num5 => "5".to_string(),
        rdev::Key::Num6 => "6".to_string(),
        rdev::Key::Num7 => "7".to_string(),
        rdev::Key::Num8 => "8".to_string(),
        rdev::Key::Num9 => "9".to_string(),
        rdev::Key::KeyA => "A".to_string(),
        rdev::Key::KeyB => "B".to_string(),
        rdev::Key::KeyC => "C".to_string(),
        rdev::Key::KeyD => "D".to_string(),
        rdev::Key::KeyE => "E".to_string(),
        rdev::Key::KeyF => "F".to_string(),
        rdev::Key::KeyG => "G".to_string(),
        rdev::Key::KeyH => "H".to_string(),
        rdev::Key::KeyI => "I".to_string(),
        rdev::Key::KeyJ => "J".to_string(),
        rdev::Key::KeyK => "K".to_string(),
        rdev::Key::KeyL => "L".to_string(),
        rdev::Key::KeyM => "M".to_string(),
        rdev::Key::KeyN => "N".to_string(),
        rdev::Key::KeyO => "O".to_string(),
        rdev::Key::KeyP => "P".to_string(),
        rdev::Key::KeyQ => "Q".to_string(),
        rdev::Key::KeyR => "R".to_string(),
        rdev::Key::KeyS => "S".to_string(),
        rdev::Key::KeyT => "T".to_string(),
        rdev::Key::KeyU => "U".to_string(),
        rdev::Key::KeyV => "V".to_string(),
        rdev::Key::KeyW => "W".to_string(),
        rdev::Key::KeyX => "X".to_string(),
        rdev::Key::KeyY => "Y".to_string(),
        rdev::Key::KeyZ => "Z".to_string(),
        rdev::Key::F1 => "F1".to_string(),
        rdev::Key::F2 => "F2".to_string(),
        rdev::Key::F3 => "F3".to_string(),
        rdev::Key::F4 => "F4".to_string(),
        rdev::Key::F5 => "F5".to_string(),
        rdev::Key::F6 => "F6".to_string(),
        rdev::Key::F7 => "F7".to_string(),
        rdev::Key::F8 => "F8".to_string(),
        rdev::Key::F9 => "F9".to_string(),
        rdev::Key::F10 => "F10".to_string(),
        rdev::Key::F11 => "F11".to_string(),
        rdev::Key::F12 => "F12".to_string(),
        _ => return None,
    };

    Some(normalized)
}
