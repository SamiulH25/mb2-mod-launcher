use crate::AppContext;
use mb2_core::{
    auto_sort, default_cache_path, launch_via_steam, load_app_state,
    save_load_order as persist_load_order, unblock_module_dlls, GamePaths, LoadOrderEntry,
    MetadataCache,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn detect_game(ctx: State<'_, AppContext>) -> Result<mb2_core::AppState, String> {
    let paths = GamePaths::detect().map_err(|e| e.to_string())?;
    let state = load_app_state(&paths).map_err(|e| e.to_string())?;

    *ctx.paths.lock().unwrap() = Some(paths);
    *ctx.state.lock().unwrap() = Some(state.clone());

    Ok(state)
}

#[tauri::command]
pub fn set_game_path(
    ctx: State<'_, AppContext>,
    game_root: String,
) -> Result<mb2_core::AppState, String> {
    let paths = GamePaths::from_game_root(PathBuf::from(game_root)).map_err(|e| e.to_string())?;
    let state = load_app_state(&paths).map_err(|e| e.to_string())?;

    *ctx.paths.lock().unwrap() = Some(paths);
    *ctx.state.lock().unwrap() = Some(state.clone());

    Ok(state)
}

#[tauri::command]
pub fn refresh_modules(ctx: State<'_, AppContext>) -> Result<mb2_core::AppState, String> {
    let paths = ctx
        .paths
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Game not configured. Detect or set game path first.".to_string())?;

    let state = load_app_state(&paths).map_err(|e| e.to_string())?;
    *ctx.state.lock().unwrap() = Some(state.clone());
    Ok(state)
}

#[tauri::command]
pub fn toggle_module(
    ctx: State<'_, AppContext>,
    module_id: String,
    enabled: bool,
) -> Result<mb2_core::AppState, String> {
    let mut state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    if let Some(module) = state.modules.iter_mut().find(|m| m.module.info.id == module_id) {
        module.enabled = enabled;
    }

    *ctx.state.lock().unwrap() = Some(state.clone());
    Ok(state)
}

#[tauri::command]
pub fn set_all_modules_enabled(
    ctx: State<'_, AppContext>,
    enabled: bool,
) -> Result<mb2_core::AppState, String> {
    let mut state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    for module in &mut state.modules {
        module.enabled = enabled;
    }

    *ctx.state.lock().unwrap() = Some(state.clone());
    Ok(state)
}

#[tauri::command]
pub fn reorder_modules(
    ctx: State<'_, AppContext>,
    module_ids: Vec<String>,
) -> Result<mb2_core::AppState, String> {
    let mut state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    for (position, id) in module_ids.iter().enumerate() {
        if let Some(module) = state.modules.iter_mut().find(|m| m.module.info.id == *id) {
            module.position = position;
        }
    }

    state
        .modules
        .sort_by_key(|m| m.position);

    *ctx.state.lock().unwrap() = Some(state.clone());
    Ok(state)
}

#[tauri::command]
pub fn auto_sort_modules(ctx: State<'_, AppContext>) -> Result<mb2_core::AppState, String> {
    let mut state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    let modules: Vec<_> = state.modules.iter().map(|m| m.module.info.clone()).collect();
    let enabled: HashMap<String, bool> = state
        .modules
        .iter()
        .map(|m| (m.module.info.id.clone(), m.enabled))
        .collect();

    let sort_result = auto_sort(&modules, &enabled).map_err(|e| e.to_string())?;
    state.warnings = sort_result.warnings;

    let position_map: HashMap<String, usize> = sort_result
        .order
        .iter()
        .enumerate()
        .map(|(i, e)| (e.module_id.clone(), i))
        .collect();

    for module in &mut state.modules {
        if let Some(pos) = position_map.get(&module.module.info.id) {
            module.position = *pos;
            if let Some(entry) = sort_result.order.iter().find(|e| e.module_id == module.module.info.id) {
                module.enabled = entry.enabled;
            }
        }
    }

    state.modules.sort_by_key(|m| m.position);

    *ctx.state.lock().unwrap() = Some(state.clone());
    Ok(state)
}

#[tauri::command]
pub fn save_load_order(ctx: State<'_, AppContext>) -> Result<(), String> {
    let paths = ctx
        .paths
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Game not configured.".to_string())?;

    let state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    let entries: Vec<LoadOrderEntry> = state
        .modules
        .iter()
        .map(|m| LoadOrderEntry {
            module_id: m.module.info.id.clone(),
            enabled: m.enabled,
        })
        .collect();

    persist_load_order(&paths, &entries).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unblock_dlls(ctx: State<'_, AppContext>) -> Result<Vec<mb2_core::UnblockResult>, String> {
    let state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    let mut results = Vec::new();
    for module in state.modules.iter().filter(|m| m.enabled) {
        let path = PathBuf::from(&module.module.path);
        let result = unblock_module_dlls(&path, &module.module.info).map_err(|e| e.to_string())?;
        results.push(result);
    }

    Ok(results)
}

#[tauri::command]
pub fn launch_game(ctx: State<'_, AppContext>) -> Result<(), String> {
    let paths = ctx
        .paths
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Game not configured.".to_string())?;

    let state = ctx
        .state
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No module state loaded.".to_string())?;

    let entries: Vec<LoadOrderEntry> = state
        .modules
        .iter()
        .map(|m| LoadOrderEntry {
            module_id: m.module.info.id.clone(),
            enabled: m.enabled,
        })
        .collect();

    persist_load_order(&paths, &entries).map_err(|e| e.to_string())?;
    launch_via_steam().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_cached_mods(query: String) -> Result<Vec<mb2_core::CachedModMetadata>, String> {
    let cache = MetadataCache::open(&default_cache_path()).map_err(|e| e.to_string())?;
    cache.search(&query).map_err(|e| e.to_string())
}
