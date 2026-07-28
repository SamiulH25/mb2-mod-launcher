// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::sync::Mutex;

pub struct AppContext {
    pub paths: Mutex<Option<mb2_core::GamePaths>>,
    pub state: Mutex<Option<mb2_core::AppState>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppContext {
            paths: Mutex::new(None),
            state: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_game,
            commands::set_game_path,
            commands::refresh_modules,
            commands::toggle_module,
            commands::set_all_modules_enabled,
            commands::reorder_modules,
            commands::auto_sort_modules,
            commands::save_load_order,
            commands::launch_game,
            commands::unblock_dlls,
            commands::search_cached_mods,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
