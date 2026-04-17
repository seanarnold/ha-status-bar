use std::sync::Arc;
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;
use tokio::sync::Notify;

mod config;
mod ha_client;

use config::Config;
use ha_client::{EntityState, EntitySummary};

fn load_config(app: &AppHandle) -> Config {
    app.store("config.json")
        .ok()
        .and_then(|s| s.get("config"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn entity_emoji(entity_id: &str, friendly_name: &str) -> &'static str {
    let s = format!("{} {}", entity_id, friendly_name).to_lowercase();
    let s = s.as_str();
    if s.contains("solar") || s.contains(" pv") { return "☀️" }
    if s.contains("battery") || s.contains("powerwall") { return "🔋" }
    if s.contains("grid") { return "⚡" }
    if s.contains("temperature") || s.contains("_temp") { return "🌡️" }
    if s.contains("humidity") { return "💧" }
    if s.contains("rain") || s.contains("precipitation") { return "🌧️" }
    if s.contains("wind") { return "💨" }
    if s.contains("door") { return "🚪" }
    if s.contains("window") || s.contains("blind") || s.contains("cover") { return "🪟" }
    if s.contains("motion") || s.contains("occupancy") { return "🚶" }
    if s.contains("power") || s.contains("watt") { return "⚡" }
    if s.contains("energy") || s.contains("kwh") { return "⚡" }
    if s.contains("lock") { return "🔒" }
    if s.contains("camera") { return "📷" }
    if s.contains("car") || s.contains("_ev") || s.contains("vehicle") { return "🚗" }
    if s.contains("gas") { return "🔥" }
    if s.contains("water") { return "💧" }
    if s.contains("person") || s.contains("presence") { return "👤" }
    if s.contains("smoke") || s.contains("co2") || s.contains("air") { return "🌫️" }
    let domain = entity_id.split('.').next().unwrap_or("");
    match domain {
        "light" => "💡",
        "switch" => "🔌",
        "cover" => "🪟",
        "climate" | "water_heater" => "🌡️",
        "media_player" => "🎵",
        "weather" => "⛅",
        "alarm_control_panel" => "🚨",
        "vacuum" => "🤖",
        "fan" => "🌀",
        "lock" => "🔒",
        "camera" => "📷",
        "device_tracker" | "person" => "👤",
        "input_boolean" | "binary_sensor" => "🔔",
        _ => "📊",
    }
}

fn fmt_value(s: &str) -> String {
    s.parse::<f64>()
        .map(|f| format!("{:.2}", f))
        .unwrap_or_else(|_| s.to_string())
}

fn load_icon() -> Option<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/32x32.png")).ok()
}

fn build_menu(
    app: &AppHandle,
    states: &[EntityState],
    autostart_enabled: bool,
) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;

    if states.is_empty() {
        menu.append(&MenuItem::with_id(
            app,
            "no_entities",
            "No entities configured",
            false,
            None::<&str>,
        )?)?;
    } else {
        for (i, s) in states.iter().enumerate() {
            let emoji = entity_emoji(&s.entity_id, &s.friendly_name);
            let unit = s.unit_of_measurement.trim();
            let val = fmt_value(&s.state);
            let label = if unit.is_empty() {
                format!("{} {}: {}", emoji, s.friendly_name, val)
            } else {
                format!("{} {}: {} {}", emoji, s.friendly_name, val, unit)
            };
            menu.append(&MenuItem::with_id(
                app,
                format!("entity_{i}"),
                label,
                false,
                None::<&str>,
            )?)?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "settings",
        "⚙ Settings...",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&CheckMenuItem::with_id(
        app,
        "autostart",
        "Launch at Login",
        true,
        autostart_enabled,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)?;

    Ok(menu)
}

fn format_title(states: &[EntityState]) -> String {
    if states.is_empty() {
        return "HA".to_string();
    }
    states
        .iter()
        .map(|s| {
            let emoji = entity_emoji(&s.entity_id, &s.friendly_name);
            let unit = s.unit_of_measurement.trim();
            let val = fmt_value(&s.state);
            if unit.is_empty() {
                format!("{} {}", emoji, val)
            } else {
                format!("{} {} {}", emoji, val, unit)
            }
        })
        .collect::<Vec<_>>()
        .join("  ")
}

#[tauri::command]
async fn get_config(app: AppHandle) -> Result<Config, String> {
    Ok(load_config(&app))
}

#[tauri::command]
async fn save_config(
    config: Config,
    app: AppHandle,
    notify: tauri::State<'_, Arc<Notify>>,
) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;
    store.set(
        "config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    notify.notify_one();
    Ok(())
}

#[tauri::command]
async fn test_connection(url: String, token: String) -> Result<bool, String> {
    ha_client::fetch_all_entities(&url, &token)
        .await
        .map(|_| true)
}

#[tauri::command]
async fn fetch_entities(url: String, token: String) -> Result<Vec<EntitySummary>, String> {
    ha_client::fetch_all_entities(&url, &token).await
}

#[tauri::command]
async fn preview_selection(states: Vec<EntityState>, app: AppHandle) -> Result<(), String> {
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    if let Ok(menu) = build_menu(&app, &states, autostart_enabled) {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
            let title = format_title(&states);
            let _ = tray.set_title(Some(&title));
            let _ = tray.set_icon(if states.is_empty() { load_icon() } else { None });
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let notify = Arc::new(Notify::new());
            app.manage(notify.clone());

            let autostart_enabled = app.handle().autolaunch().is_enabled().unwrap_or(false);
            let initial_menu = build_menu(app.handle(), &[], autostart_enabled)?;

            let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("tray icon load failed");

            TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .menu(&initial_menu)
                .title("HA")
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(win) = app.get_webview_window("settings") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "autostart" => {
                        let mgr = app.autolaunch();
                        if mgr.is_enabled().unwrap_or(false) {
                            let _ = mgr.disable();
                        } else {
                            let _ = mgr.enable();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // Intercept window X button — hide rather than destroy
            if let Some(win) = app.get_webview_window("settings") {
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let config = load_config(&app_handle);
                    let interval = Duration::from_secs(config.refresh_interval_secs.max(5));

                    if !config.ha_token.is_empty() && !config.selected_entities.is_empty() {
                        let autostart_enabled =
                            app_handle.autolaunch().is_enabled().unwrap_or(false);
                        match ha_client::fetch_selected(
                            &config.ha_url,
                            &config.ha_token,
                            &config.selected_entities,
                        )
                        .await
                        {
                            Ok(states) => {
                                if let Ok(menu) =
                                    build_menu(&app_handle, &states, autostart_enabled)
                                {
                                    if let Some(tray) = app_handle.tray_by_id("main") {
                                        let _ = tray.set_menu(Some(menu));
                                        let title = format_title(&states);
                                        let _ = tray.set_title(Some(&title));
                                        let _ = tray.set_icon(None::<Image>);
                                    }
                                }
                            }
                            Err(_) => {
                                if let Some(tray) = app_handle.tray_by_id("main") {
                                    let _ = tray.set_icon(load_icon());
                                    let _ = tray.set_title(Some("HA ⚠"));
                                }
                            }
                        }
                    }

                    let notify = app_handle.state::<Arc<Notify>>();
                    tokio::select! {
                        _ = tokio::time::sleep(interval) => {},
                        _ = notify.notified() => {},
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            test_connection,
            fetch_entities,
            preview_selection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
