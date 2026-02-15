#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, CustomMenuItem, Menu, MenuItem, Submenu, WindowBuilder, WindowUrl};
use std::fs;
use std::path::PathBuf;
use std::io::Write;

#[derive(serde::Serialize, serde::Deserialize)]
struct AppConfig {
    url: String,
}

fn get_config_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    app_handle.path_resolver()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
}

fn save_url(app_handle: &tauri::AppHandle, url: &str) {
    if let Some(config_path) = get_config_path(app_handle) {
        if let Some(config_dir) = config_path.parent() {
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
        }
        let config = AppConfig { url: url.to_string() };
        if let Ok(config_str) = serde_json::to_string(&config) {
            let _ = fs::write(config_path, config_str);
        }
    }
}

fn load_url(app_handle: &tauri::AppHandle) -> String {
    if let Some(config_path) = get_config_path(app_handle) {
         if config_path.exists() {
            if let Ok(config_str) = fs::read_to_string(config_path) {
                 if let Ok(config) = serde_json::from_str::<AppConfig>(&config_str) {
                    return config.url;
                }
            }
        }
    }
    "https://smartyapp.piltismart.com".to_string()
}

#[tauri::command]
fn get_current_url(app_handle: tauri::AppHandle) -> String {
    load_url(&app_handle)
}

#[tauri::command]
fn update_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    // Validate URL
    let _parsed_url: url::Url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    
    // Save the URL
    save_url(&app_handle, &url);
    
    // Use Tauri's native restart to refresh the app with the new config
    tauri::api::process::restart(&app_handle.env());
    
    Ok(())
}

fn open_settings_window(app_handle: &tauri::AppHandle) {
    // Check if settings window already exists
    if let Some(settings_window) = app_handle.get_window("settings") {
        let _ = settings_window.set_focus();
        return;
    }
    
    // Create new settings window
    let _settings_window = WindowBuilder::new(
        app_handle,
        "settings",
        WindowUrl::App("settings.html".into())
    )
    .title("Change Server")
    .inner_size(420.0, 280.0)
    .resizable(false)
    .center()
    .focused(true)
    .build();
}

fn log_debug(msg: &str) {
    let mut path = std::env::temp_dir();
    path.push("pilti_debug.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", msg);
    }
}

fn main() {
    log_debug("Starting Pilti application...");

    // App Menu (Pilti)
    let mut app_menu_inner = Menu::new()
        .add_native_item(MenuItem::About("Pilti".to_string(), tauri::AboutMetadata::default()))
        .add_native_item(MenuItem::Separator);

    #[cfg(target_os = "macos")]
    {
        app_menu_inner = app_menu_inner
            .add_native_item(MenuItem::Hide)
            .add_native_item(MenuItem::HideOthers)
            .add_native_item(MenuItem::ShowAll)
            .add_native_item(MenuItem::Separator);
    }

    app_menu_inner = app_menu_inner.add_native_item(MenuItem::Quit);
    let app_menu = Submenu::new("Pilti", app_menu_inner);

    // Settings Menu
    let change_url = CustomMenuItem::new("change_url".to_string(), "Change Server").accelerator("CmdOrCtrl+Shift+C");
    let settings_menu = Submenu::new("Settings", Menu::new().add_item(change_url));

    let menu = Menu::new()
        .add_submenu(app_menu)
        .add_submenu(settings_menu);

    log_debug("Menu created, building app...");

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| {
            if event.menu_item_id() == "change_url" {
                let app_handle = event.window().app_handle();
                open_settings_window(&app_handle);
            }
        })
        .setup(|app| {
            log_debug("App setup started");
            let app_handle = app.handle();
            let url = load_url(&app_handle);
            log_debug(&format!("Loaded URL: {}", url));
            
            if let Some(main_window) = app.get_window("main") {
                log_debug("Main window found, redirecting...");
                // Redirect to saved URL or default
                let script = format!("window.location.href = '{}';", url);
                let _ = main_window.eval(&script);
            } else {
                log_debug("Main window NOT found via get_window");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_current_url, update_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
