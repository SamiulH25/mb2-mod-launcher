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

/// Returns `steamapps/workshop/content/261550` from each installed Steam library.
pub fn discover_workshop_dirs() -> Result<Vec<PathBuf>> {
    let libraries = discover_steam_libraries()?;
    let mut dirs = Vec::new();

    for library in libraries {
        let workshop = library
            .join("steamapps/workshop/content")
            .join(STEAM_APP_ID);
        if workshop.is_dir() {
            dirs.push(workshop);
        }
    }

    dirs.sort();
    dirs.dedup();
    Ok(dirs)
}

impl GamePaths {
    pub fn detect() -> Result<Self> {
        let steam_libraries = discover_steam_libraries()?;
        for library in &steam_libraries {
            let game_root = library
                .join("steamapps/common")
                .join(GAME_FOLDER_NAME);
            if game_root.is_dir() {
                let modules_dir = game_root.join("Modules");
                let workshop_dirs = discover_workshop_dirs()?;
                let workshop_dir = library
                    .join("steamapps/workshop/content")
                    .join(STEAM_APP_ID);
                let workshop_dir = workshop_dir.is_dir().then_some(workshop_dir);

                let proton_prefix = library
                    .parent()
                    .map(|p| p.join("compatdata").join(STEAM_APP_ID).join("pfx"));

                let launcher_data = proton_prefix
                    .as_ref()
                    .map(|pfx| {
                        pfx.join("drive_c/users/steamuser/Documents")
                            .join(DOCUMENTS_FOLDER_NAME)
                            .join("Configs")
                            .join(CONFIG_FILE_NAME)
                    })
                    .filter(|p| p.exists())
                    .or_else(|| {
                        let home = std::env::var_os("HOME").map(PathBuf::from)?;
                        let wine_docs = home
                            .join(".steam/steam/steamapps/compatdata")
                            .join(STEAM_APP_ID)
                            .join("pfx/drive_c/users/steamuser/Documents")
                            .join(DOCUMENTS_FOLDER_NAME)
                            .join("Configs")
                            .join(CONFIG_FILE_NAME);
                        wine_docs.exists().then_some(wine_docs)
                    })
                    .unwrap_or_else(|| {
                        PathBuf::from("/tmp/mb2-mod-launcher/LauncherData.xml")
                    });

                return Ok(Self {
                    game_root,
                    modules_dir,
                    launcher_data,
                    workshop_dir,
                    workshop_dirs,
                    proton_prefix: proton_prefix.filter(|p| p.is_dir()),
                    steam_library: library.clone(),
                });
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

        let modules_dir = game_root.join("Modules");
        let workshop_dirs = discover_workshop_dirs().unwrap_or_default();
        let workshop_dir = steam_library
            .join("steamapps/workshop/content")
            .join(STEAM_APP_ID);
        let workshop_dir = workshop_dir.is_dir().then_some(workshop_dir);

        let proton_prefix = steam_library
            .parent()
            .map(|p| p.join("compatdata").join(STEAM_APP_ID).join("pfx"));

        let launcher_data = proton_prefix
            .as_ref()
            .map(|pfx| {
                pfx.join("drive_c/users/steamuser/Documents")
                    .join(DOCUMENTS_FOLDER_NAME)
                    .join("Configs")
                    .join(CONFIG_FILE_NAME)
            })
            .unwrap_or_else(|| PathBuf::from("/tmp/mb2-mod-launcher/LauncherData.xml"));

        Ok(Self {
            game_root,
            modules_dir,
            launcher_data,
            workshop_dir,
            workshop_dirs,
            proton_prefix: proton_prefix.filter(|p| p.is_dir()),
            steam_library,
        })
    }
}

fn discover_steam_libraries() -> Result<Vec<PathBuf>> {
    let mut libraries = Vec::new();

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        let default = home.join(".local/share/Steam");
        if default.is_dir() {
            libraries.push(default.clone());
            libraries.extend(parse_libraryfolders_vdf(&default)?);
        }

        let steam_compat = home.join(".steam/steam");
        if steam_compat.is_dir() && !libraries.contains(&steam_compat) {
            libraries.push(steam_compat.clone());
            libraries.extend(parse_libraryfolders_vdf(&steam_compat)?);
        }
    }

    libraries.sort();
    libraries.dedup();
    libraries.retain(|p| p.is_dir());

    if libraries.is_empty() {
        return Err(Mb2Error::GameNotFound(
            "No Steam installation found on this system.".into(),
        ));
    }

    Ok(libraries)
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
}
