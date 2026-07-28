use crate::error::{Mb2Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const STEAM_APP_ID: &str = "261550";
pub const GAME_FOLDER_NAME: &str = "Mount & Blade II Bannerlord";
pub const DOCUMENTS_FOLDER_NAME: &str = "Mount and Blade II Bannerlord";
pub const CONFIG_FILE_NAME: &str = "LauncherData.xml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePaths {
    pub game_root: PathBuf,
    pub modules_dir: PathBuf,
    pub launcher_data: PathBuf,
    /// Primary workshop folder (same Steam library as the game install, if present).
    pub workshop_dir: Option<PathBuf>,
    /// Workshop folders from every Steam library (mods may download to a different drive).
    pub workshop_dirs: Vec<PathBuf>,
    pub proton_prefix: Option<PathBuf>,
    pub steam_library: PathBuf,
}

/// Returns `steamapps/workshop/content/261550` from every known Steam library.
///
/// `game_library` is always included even when it is missing from `libraryfolders.vdf`
/// (e.g. external drives mounted after Steam started).
pub fn discover_workshop_dirs(game_library: Option<&Path>) -> Result<Vec<PathBuf>> {
    let libraries = discover_steam_libraries()?;
    workshop_dirs_from_libraries(&libraries, game_library)
}

pub fn workshop_dirs_from_libraries(
    libraries: &[PathBuf],
    game_library: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();

    for library in libraries {
        push_workshop_dir_for_library(library, &mut dirs);
    }

    if let Some(game_library) = game_library {
        push_workshop_dir_for_library(game_library, &mut dirs);
    }

    Ok(dedup_paths(dirs))
}

fn push_workshop_dir_for_library(library: &Path, dirs: &mut Vec<PathBuf>) {
    let workshop = library
        .join("steamapps/workshop/content")
        .join(STEAM_APP_ID);
    if workshop.is_dir() {
        dirs.push(workshop);
    }
}

/// Returns `steamapps/compatdata/261550` from the first library that has a Proton prefix.
pub fn discover_proton_compat_data(libraries: &[PathBuf]) -> Option<PathBuf> {
    for library in libraries {
        let compat = library
            .join("steamapps/compatdata")
            .join(STEAM_APP_ID);
        if compat.join("pfx").is_dir() {
            return Some(compat);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        for steam_root in [home.join(".steam/steam"), home.join(".local/share/Steam")] {
            let compat = steam_root
                .join("steamapps/compatdata")
                .join(STEAM_APP_ID);
            if compat.join("pfx").is_dir() {
                return Some(compat);
            }
        }
    }

    None
}

pub fn discover_proton_prefix(libraries: &[PathBuf]) -> Option<PathBuf> {
    discover_proton_compat_data(libraries).map(|compat| compat.join("pfx"))
}

pub fn launcher_data_path_for_prefix(prefix: &Path) -> PathBuf {
    prefix
        .join("drive_c/users/steamuser/Documents")
        .join(DOCUMENTS_FOLDER_NAME)
        .join("Configs")
        .join(CONFIG_FILE_NAME)
}

pub fn launcher_data_path_for_library(steam_library: &Path) -> PathBuf {
    launcher_data_path_for_prefix(
        &steam_library
            .join("steamapps/compatdata")
            .join(STEAM_APP_ID)
            .join("pfx"),
    )
}

fn build_paths_for_game(game_root: PathBuf, steam_library: PathBuf) -> Result<GamePaths> {
    let libraries = discover_steam_libraries().unwrap_or_default();
    // Prefer the Proton prefix beside the game install (external libraries keep their own compatdata).
    let library_prefix = steam_library
        .join("steamapps/compatdata")
        .join(STEAM_APP_ID)
        .join("pfx");
    let proton_prefix = library_prefix
        .is_dir()
        .then_some(library_prefix)
        .or_else(|| discover_proton_prefix(&libraries));

    let launcher_data = proton_prefix
        .as_ref()
        .map(|pfx| launcher_data_path_for_prefix(pfx.as_path()))
        .unwrap_or_else(|| launcher_data_path_for_library(&steam_library));

    let workshop_dirs =
        discover_workshop_dirs(Some(steam_library.as_path())).unwrap_or_default();
    let workshop_dir = steam_library
        .join("steamapps/workshop/content")
        .join(STEAM_APP_ID);
    let workshop_dir = workshop_dir.is_dir().then_some(workshop_dir);

    Ok(GamePaths {
        modules_dir: game_root.join("Modules"),
        launcher_data,
        workshop_dir,
        workshop_dirs,
        proton_prefix,
        steam_library,
        game_root,
    })
}

impl GamePaths {
    /// Re-scan Steam libraries and workshop folders (call on refresh after new downloads).
    pub fn refresh_discovered_paths(&mut self) -> Result<()> {
        let libraries = discover_steam_libraries()?;
        self.workshop_dirs =
            discover_workshop_dirs(Some(self.steam_library.as_path()))?;
        let workshop_candidate = self
            .steam_library
            .join("steamapps/workshop/content")
            .join(STEAM_APP_ID);
        self.workshop_dir = workshop_candidate
            .is_dir()
            .then_some(workshop_candidate);

        let library_prefix = self
            .steam_library
            .join("steamapps/compatdata")
            .join(STEAM_APP_ID)
            .join("pfx");
        self.proton_prefix = library_prefix
            .is_dir()
            .then_some(library_prefix)
            .or_else(|| discover_proton_prefix(&libraries));

        self.launcher_data = self
            .proton_prefix
            .as_ref()
            .map(|pfx| launcher_data_path_for_prefix(pfx.as_path()))
            .unwrap_or_else(|| launcher_data_path_for_library(&self.steam_library));

        Ok(())
    }

    pub fn detect() -> Result<Self> {
        let steam_libraries = discover_steam_libraries()?;
        for library in &steam_libraries {
            let game_root = library
                .join("steamapps/common")
                .join(GAME_FOLDER_NAME);
            if game_root.is_dir() {
                return build_paths_for_game(game_root, library.clone());
            }
        }

        Err(Mb2Error::GameNotFound(
            "Could not find Bannerlord in any Steam library. Is it installed via Steam?".into(),
        ))
    }

    pub fn from_game_root(game_root: PathBuf) -> Result<Self> {
        if !game_root.is_dir() {
            return Err(Mb2Error::GameNotFound(format!(
                "Game path does not exist: {}",
                game_root.display()
            )));
        }

        let steam_library = game_root
            .ancestors()
            .find(|p| p.ends_with("common"))
            .and_then(|common| common.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| game_root.clone());

        build_paths_for_game(game_root, steam_library)
    }
}

pub fn discover_steam_libraries() -> Result<Vec<PathBuf>> {
    let mut libraries = Vec::new();

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(&home);
        let steam_roots = [
            home.join(".local/share/Steam"),
            home.join(".steam/steam"),
            home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
        ];

        for steam_root in steam_roots {
            if !steam_root.is_dir() {
                continue;
            }
            push_unique_library(&mut libraries, steam_root.clone());
            for library in parse_libraryfolders_vdf(&steam_root)? {
                push_unique_library(&mut libraries, library);
            }
        }
    }

    libraries.retain(|p| p.is_dir());

    if libraries.is_empty() {
        return Err(Mb2Error::GameNotFound(
            "No Steam installation found on this system.".into(),
        ));
    }

    Ok(libraries)
}

fn push_unique_library(libraries: &mut Vec<PathBuf>, candidate: PathBuf) {
    if libraries.iter().any(|existing| paths_equal(existing, &candidate)) {
        return;
    }
    libraries.push(candidate);
}

/// Best-effort canonical compare so duplicate mount paths (label vs UUID) collapse.
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }

    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a.to_string_lossy() == b.to_string_lossy(),
    }
}

pub fn dedup_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique: Vec<PathBuf> = Vec::new();
    for path in paths {
        if !unique
            .iter()
            .any(|existing| paths_equal(existing.as_path(), path.as_path()))
        {
            unique.push(path);
        }
    }
    unique.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    unique
}

fn parse_libraryfolders_vdf(steam_root: &Path) -> Result<Vec<PathBuf>> {
    let vdf_path = steam_root.join("steamapps/libraryfolders.vdf");
    if !vdf_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&vdf_path)?;
    let mut paths = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("\"path\"") {
            if let Some(path_str) = extract_vdf_string_value(rest) {
                paths.push(PathBuf::from(path_str));
            }
        }
    }

    Ok(paths)
}

fn extract_vdf_string_value(rest: &str) -> Option<String> {
    let mut parts = rest.split('"').filter(|s| !s.trim().is_empty());
    parts.next().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vdf_path_line() {
        let value = extract_vdf_string_value("\t\t\"/mnt/games/steam\"");
        assert_eq!(value.as_deref(), Some("/mnt/games/steam"));
    }

    #[test]
    fn launcher_data_uses_steamapps_compatdata() {
        let library = PathBuf::from("/steam/SteamLibrary");
        let path = launcher_data_path_for_library(&library);
        assert!(path.to_string_lossy().contains("steamapps/compatdata/261550/pfx"));
        assert!(path.to_string_lossy().ends_with("LauncherData.xml"));
    }

    #[test]
    fn dedup_paths_collapses_duplicates() {
        let paths = dedup_paths(vec![
            PathBuf::from("/steam/A"),
            PathBuf::from("/steam/A"),
            PathBuf::from("/steam/B"),
        ]);
        assert_eq!(paths.len(), 2);
    }
}
