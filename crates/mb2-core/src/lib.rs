pub mod cache;
pub mod dll;
pub mod error;
pub mod launcher_data;
pub mod launch;
pub mod load_order;
pub mod paths;
pub mod scanner;
pub mod submodule;

pub use cache::{CachedModMetadata, MetadataCache};
pub use dll::{unblock_module_dlls, UnblockResult};
pub use error::{Mb2Error, Result};
pub use launcher_data::{read_launcher_data, write_launcher_data, LauncherData};
pub use launch::launch_via_steam;
pub use load_order::{auto_sort, is_official_module, LoadOrderEntry, SortResult};
pub use paths::GamePaths;
pub use scanner::{scan_modules, InstalledModule, ModuleSource};
pub use submodule::SubModuleInfo;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleState {
    pub module: InstalledModule,
    pub enabled: bool,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub paths: GamePathsSnapshot,
    pub modules: Vec<ModuleState>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePathsSnapshot {
    pub game_root: String,
    pub modules_dir: String,
    pub launcher_data: String,
    pub workshop_dir: Option<String>,
    pub workshop_dirs: Vec<String>,
    pub proton_prefix: Option<String>,
}

impl From<GamePaths> for GamePathsSnapshot {
    fn from(paths: GamePaths) -> Self {
        Self {
            game_root: paths.game_root.to_string_lossy().into_owned(),
            modules_dir: paths.modules_dir.to_string_lossy().into_owned(),
            launcher_data: paths.launcher_data.to_string_lossy().into_owned(),
            workshop_dir: paths
                .workshop_dir
                .map(|p| p.to_string_lossy().into_owned()),
            workshop_dirs: paths
                .workshop_dirs
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
            proton_prefix: paths
                .proton_prefix
                .map(|p| p.to_string_lossy().into_owned()),
        }
    }
}

pub fn load_app_state(paths: &GamePaths) -> Result<AppState> {
    let installed = scan_modules(paths)?;
    let launcher = read_launcher_data(&paths.launcher_data)?;

    let launcher_map: HashMap<String, bool> = launcher
        .singleplayer
        .iter()
        .map(|e| (e.module_id.clone(), e.enabled))
        .collect();

    let position_map: HashMap<String, usize> = launcher
        .singleplayer
        .iter()
        .enumerate()
        .map(|(i, e)| (e.module_id.clone(), i))
        .collect();

    let mut modules: Vec<ModuleState> = installed
        .into_iter()
        .map(|module| {
            let enabled = launcher_map
                .get(&module.info.id)
                .copied()
                .unwrap_or(is_official_module(&module.info.id));
            let position = position_map
                .get(&module.info.id)
                .copied()
                .unwrap_or(usize::MAX);
            ModuleState {
                module,
                enabled,
                position,
            }
        })
        .collect();

    modules.sort_by_key(|m| m.position);

    Ok(AppState {
        paths: paths.clone().into(),
        modules,
        warnings: Vec::new(),
    })
}

pub fn save_load_order(paths: &GamePaths, entries: &[LoadOrderEntry]) -> Result<()> {
    let existing = read_launcher_data(&paths.launcher_data)?;
    let data = LauncherData {
        singleplayer: entries.to_vec(),
        dll_check_data: existing.dll_check_data,
    };
    write_launcher_data(&paths.launcher_data, &data)
}

pub fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mb2-mod-launcher")
}

pub fn default_cache_path() -> PathBuf {
    default_config_dir().join("metadata.db")
}

// Re-export dirs helper - add dirs crate
mod dirs {
    use std::path::PathBuf;

    pub fn config_dir() -> Option<PathBuf> {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
            })
    }
}
