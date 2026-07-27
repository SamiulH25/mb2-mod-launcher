use crate::error::Result;
use crate::paths::{discover_workshop_dirs, GamePaths};
use crate::submodule::{parse_submodule_file, SubModuleInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How deep to search inside a workshop item folder for `SubModule.xml`.
const WORKSHOP_SEARCH_DEPTH: u32 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModule {
    pub info: SubModuleInfo,
    pub path: String,
    pub source: ModuleSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSource {
    Game,
    Workshop,
    Manual,
}

pub fn scan_modules(paths: &GamePaths) -> Result<Vec<InstalledModule>> {
    let mut by_id: HashMap<String, InstalledModule> = HashMap::new();

    if paths.modules_dir.is_dir() {
        scan_flat_modules_dir(&paths.modules_dir, ModuleSource::Game, &mut by_id)?;
    }

    let workshop_dirs = if paths.workshop_dirs.is_empty() {
        discover_workshop_dirs()?
    } else {
        paths.workshop_dirs.clone()
    };

    for workshop_dir in workshop_dirs {
        if workshop_dir.is_dir() {
            scan_workshop_root(&workshop_dir, &mut by_id)?;
        }
    }

    let mut modules: Vec<_> = by_id.into_values().collect();
    modules.sort_by(|a, b| a.info.name.cmp(&b.info.name));
    Ok(modules)
}

fn scan_flat_modules_dir(
    dir: &Path,
    source: ModuleSource,
    by_id: &mut HashMap<String, InstalledModule>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        try_register_module_dir(&entry.path(), source.clone(), by_id);
    }

    Ok(())
}

fn scan_workshop_root(
    workshop_dir: &Path,
    by_id: &mut HashMap<String, InstalledModule>,
) -> Result<()> {
    for entry in std::fs::read_dir(workshop_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            scan_workshop_item(&entry.path(), by_id)?;
        }
    }

    Ok(())
}

fn scan_workshop_item(item_dir: &Path, by_id: &mut HashMap<String, InstalledModule>) -> Result<()> {
    let mut submodule_files = Vec::new();
    collect_submodule_files(item_dir, WORKSHOP_SEARCH_DEPTH, &mut submodule_files)?;

    for submodule_path in submodule_files {
        let module_dir = match submodule_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => continue,
        };
        try_register_module_dir(&module_dir, ModuleSource::Workshop, by_id);
    }

    Ok(())
}

fn collect_submodule_files(
    dir: &Path,
    depth: u32,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    let submodule_path = dir.join("SubModule.xml");
    if submodule_path.is_file() {
        out.push(submodule_path);
        return Ok(());
    }

    if depth == 0 {
        return Ok(());
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            tracing::warn!("Could not read {}: {}", dir.display(), err);
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                tracing::warn!("Could not read entry in {}: {}", dir.display(), err);
                continue;
            }
        };

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                tracing::warn!(
                    "Could not inspect {}: {}",
                    entry.path().display(),
                    err
                );
                continue;
            }
        };

        if file_type.is_dir() || file_type.is_symlink() {
            collect_submodule_files(&entry.path(), depth - 1, out)?;
        }
    }

    Ok(())
}

fn try_register_module_dir(
    module_dir: &Path,
    source: ModuleSource,
    by_id: &mut HashMap<String, InstalledModule>,
) {
    let submodule_path = module_dir.join("SubModule.xml");
    if !submodule_path.is_file() {
        return;
    }

    match parse_submodule_file(&submodule_path) {
        Ok(info) => {
            let candidate = InstalledModule {
                info,
                path: module_dir.to_string_lossy().into_owned(),
                source,
            };
            insert_module(by_id, candidate);
        }
        Err(err) => {
            tracing::warn!("Skipping {}: {}", module_dir.display(), err);
        }
    }
}

fn insert_module(by_id: &mut HashMap<String, InstalledModule>, candidate: InstalledModule) {
    match by_id.get(&candidate.info.id) {
        Some(existing) if !should_replace(existing, &candidate) => {}
        _ => {
            by_id.insert(candidate.info.id.clone(), candidate);
        }
    }
}

fn should_replace(existing: &InstalledModule, candidate: &InstalledModule) -> bool {
    match (&existing.source, &candidate.source) {
        (ModuleSource::Game, _) => false,
        (_, ModuleSource::Game) => true,
        (ModuleSource::Workshop, ModuleSource::Workshop) => {
            candidate.path.len() < existing.path.len()
        }
        (ModuleSource::Manual, ModuleSource::Manual) => {
            candidate.path.len() < existing.path.len()
        }
        (ModuleSource::Manual, ModuleSource::Workshop) => false,
        (ModuleSource::Workshop, ModuleSource::Manual) => true,
    }
}

pub fn enabled_map_from_launcher(
    module_ids: &[String],
    launcher_entries: &[(String, bool)],
) -> HashMap<String, bool> {
    let launcher_map: HashMap<_, _> = launcher_entries.iter().cloned().collect();

    module_ids
        .iter()
        .map(|id| {
            let enabled = launcher_map.get(id).copied().unwrap_or(false);
            (id.clone(), enabled)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SAMPLE_SUBMODULE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Module>
  <Name value="Test Mod"/>
  <Id value="Test.Mod"/>
  <Version value="v1.0.0"/>
  <SingleplayerModule value="true"/>
</Module>"#;

    fn write_test_module(dir: &Path, id: &str) {
        fs::create_dir_all(dir).unwrap();
        let xml = SAMPLE_SUBMODULE.replace("Test.Mod", id).replace("Test Mod", id);
        fs::write(dir.join("SubModule.xml"), xml).unwrap();
    }

    fn scan_temp_workshop(
        test_name: &str,
        layout: impl FnOnce(&Path),
    ) -> Vec<InstalledModule> {
        let root = std::env::temp_dir().join(format!(
            "mb2-scanner-test-{}-{}",
            test_name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        layout(&root);

        let mut by_id = HashMap::new();
        for entry in fs::read_dir(&root).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                scan_workshop_item(&entry.path(), &mut by_id).unwrap();
            }
        }

        let _ = fs::remove_dir_all(&root);
        by_id.into_values().collect()
    }

    #[test]
    fn detects_submodule_at_workshop_item_root() {
        let modules = scan_temp_workshop("root", |root| {
            write_test_module(&root.join("1234567890"), "Root.Mod");
        });

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].info.id, "Root.Mod");
        assert_eq!(modules[0].source, ModuleSource::Workshop);
    }

    #[test]
    fn detects_nested_workshop_module_folder() {
        let modules = scan_temp_workshop("nested", |root| {
            write_test_module(&root.join("1234567890").join("MyCustomMod"), "Nested.Mod");
        });

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].info.id, "Nested.Mod");
    }

    #[test]
    fn detects_modules_subdirectory_layout() {
        let modules = scan_temp_workshop("modules-subdir", |root| {
            write_test_module(
                &root.join("1234567890").join("Modules").join("OpenSourceArmory"),
                "Modules.Subdir.Mod",
            );
        });

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].info.id, "Modules.Subdir.Mod");
    }

    #[test]
    fn prefers_game_install_over_workshop_duplicate() {
        let mut by_id = HashMap::new();
        insert_module(
            &mut by_id,
            InstalledModule {
                info: SubModuleInfo {
                    id: "Shared.Mod".into(),
                    name: "Shared".into(),
                    version: None,
                    singleplayer: true,
                    multiplayer: false,
                    depended_modules: Vec::new(),
                    depended_module_metadatas: Vec::new(),
                    dll_names: Vec::new(),
                    url: None,
                    folder_name: "Shared".into(),
                },
                path: "/workshop/content/261550/1/Shared".into(),
                source: ModuleSource::Workshop,
            },
        );
        insert_module(
            &mut by_id,
            InstalledModule {
                info: SubModuleInfo {
                    id: "Shared.Mod".into(),
                    name: "Shared".into(),
                    version: None,
                    singleplayer: true,
                    multiplayer: false,
                    depended_modules: Vec::new(),
                    depended_module_metadatas: Vec::new(),
                    dll_names: Vec::new(),
                    url: None,
                    folder_name: "Shared".into(),
                },
                path: "/game/Modules/Shared".into(),
                source: ModuleSource::Game,
            },
        );

        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id["Shared.Mod"].source, ModuleSource::Game);
    }
}
