use crate::error::Result;
use crate::load_order::is_official_module;
use crate::paths::{discover_workshop_dirs, GamePaths};
use crate::submodule::{parse_submodule_file, SubModuleInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How deep to search inside a workshop item folder for `SubModule.xml`.
const WORKSHOP_SEARCH_DEPTH: u32 = 8;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    pub workshop_dirs_scanned: usize,
    pub workshop_items_scanned: usize,
    pub modules_game: usize,
    pub modules_workshop: usize,
    pub parse_failures: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModule {
    pub info: SubModuleInfo,
    pub path: String,
    pub source: ModuleSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workshop_item_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSource {
    Game,
    Workshop,
    Manual,
}

pub fn scan_modules(paths: &GamePaths) -> Result<Vec<InstalledModule>> {
    let (modules, _) = scan_modules_with_report(paths)?;
    Ok(modules)
}

pub fn scan_modules_with_report(paths: &GamePaths) -> Result<(Vec<InstalledModule>, ScanReport)> {
    let mut report = ScanReport::default();
    let mut by_id: HashMap<String, InstalledModule> = HashMap::new();

    // Workshop first so community duplicates in `Modules/` are replaced by the live Workshop copy.
    let workshop_dirs = discover_workshop_dirs(Some(paths.steam_library.as_path()))?;
    report.workshop_dirs_scanned = workshop_dirs.len();

    if workshop_dirs.is_empty() {
        report.warnings.push(
            "No Bannerlord workshop folders found. Subscribed mods may be on an unmounted Steam library."
                .into(),
        );
    }

    for workshop_dir in &workshop_dirs {
        if workshop_dir.is_dir() {
            scan_workshop_root(workshop_dir, &mut by_id, &mut report)?;
        }
    }

    if paths.modules_dir.is_dir() {
        scan_flat_modules_dir(&paths.modules_dir, ModuleSource::Game, &mut by_id, &mut report)?;
    }

    let mut modules: Vec<_> = by_id.into_values().collect();
    modules.sort_by(|a, b| a.info.name.cmp(&b.info.name));

    report.modules_game = modules
        .iter()
        .filter(|m| m.source == ModuleSource::Game)
        .count();
    report.modules_workshop = modules
        .iter()
        .filter(|m| m.source == ModuleSource::Workshop)
        .count();

    Ok((modules, report))
}

fn scan_flat_modules_dir(
    dir: &Path,
    source: ModuleSource,
    by_id: &mut HashMap<String, InstalledModule>,
    report: &mut ScanReport,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        try_register_module_dir(&entry.path(), source.clone(), None, by_id, report);
    }

    Ok(())
}

fn scan_workshop_root(
    workshop_dir: &Path,
    by_id: &mut HashMap<String, InstalledModule>,
    report: &mut ScanReport,
) -> Result<()> {
    for entry in std::fs::read_dir(workshop_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        report.workshop_items_scanned += 1;
        let item_id = entry.file_name().to_string_lossy().into_owned();
        scan_workshop_item(&entry.path(), &item_id, by_id, report)?;
    }

    Ok(())
}

fn scan_workshop_item(
    item_dir: &Path,
    workshop_item_id: &str,
    by_id: &mut HashMap<String, InstalledModule>,
    report: &mut ScanReport,
) -> Result<()> {
    let mut submodule_files = Vec::new();
    collect_submodule_files(item_dir, WORKSHOP_SEARCH_DEPTH, &mut submodule_files)?;

    if submodule_files.is_empty() {
        report.warnings.push(format!(
            "Workshop item {} has no SubModule.xml (searched {} levels deep)",
            workshop_item_id, WORKSHOP_SEARCH_DEPTH
        ));
        return Ok(());
    }

    for submodule_path in submodule_files {
        let module_dir = match submodule_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => continue,
        };
        try_register_module_dir(
            &module_dir,
            ModuleSource::Workshop,
            Some(workshop_item_id.to_string()),
            by_id,
            report,
        );
    }

    Ok(())
}

fn collect_submodule_files(
    dir: &Path,
    depth: u32,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    if let Some(submodule_path) = submodule_xml_in_dir(dir) {
        out.push(submodule_path);
        // Keep searching children — some collection packs ship a root manifest plus nested modules.
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

fn submodule_xml_in_dir(dir: &Path) -> Option<PathBuf> {
    let direct = dir.join("SubModule.xml");
    if direct.is_file() {
        return Some(direct);
    }

    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        if file_name.eq_ignore_ascii_case("SubModule.xml") {
            let path = entry.path();
            if path.is_file() {
                return Some(path);
            }
        }
    }

    None
}

fn try_register_module_dir(
    module_dir: &Path,
    source: ModuleSource,
    workshop_item_id: Option<String>,
    by_id: &mut HashMap<String, InstalledModule>,
    report: &mut ScanReport,
) {
    let submodule_path = match submodule_xml_in_dir(module_dir) {
        Some(path) => path,
        None => return,
    };

    match parse_submodule_file(&submodule_path) {
        Ok(info) => {
            let candidate = InstalledModule {
                info,
                path: module_dir.to_string_lossy().into_owned(),
                source,
                workshop_item_id,
            };
            insert_module(by_id, candidate);
        }
        Err(err) => {
            let message = format!("Skipping {}: {}", module_dir.display(), err);
            tracing::warn!("{}", message);
            report.parse_failures.push(message);
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
    let id = &existing.info.id;

    match (&existing.source, &candidate.source) {
        // Official vanilla modules: game install wins over workshop duplicates.
        (ModuleSource::Game, ModuleSource::Workshop) if is_official_module(id) => false,
        (ModuleSource::Workshop, ModuleSource::Game) if is_official_module(id) => true,

        // Community mods: prefer the Workshop subscription over a stale `Modules/` copy.
        (ModuleSource::Game, ModuleSource::Workshop) if !is_official_module(id) => true,
        (ModuleSource::Workshop, ModuleSource::Game) if !is_official_module(id) => false,

        (ModuleSource::Workshop, ModuleSource::Workshop) => {
            prefer_workshop_candidate(existing, candidate)
        }
        (ModuleSource::Manual, ModuleSource::Manual) => {
            prefer_workshop_candidate(existing, candidate)
        }
        (ModuleSource::Manual, ModuleSource::Workshop) => true,
        (ModuleSource::Workshop, ModuleSource::Manual) => false,
        _ => false,
    }
}

fn prefer_workshop_candidate(existing: &InstalledModule, candidate: &InstalledModule) -> bool {
    match (
        workshop_item_id_from_path(&existing.path),
        workshop_item_id_from_path(&candidate.path),
    ) {
        (Some(a), Some(b)) if b != a => return b > a,
        _ => {}
    }

    submodule_mtime(&candidate.path) > submodule_mtime(&existing.path)
}

fn workshop_item_id_from_path(path: &str) -> Option<u64> {
    let marker = "/workshop/content/261550/";
    let after = path.split(marker).nth(1)?;
    after.split('/').next()?.parse().ok()
}

fn submodule_mtime(path: &str) -> u64 {
    let submodule = Path::new(path).join("SubModule.xml");
    std::fs::metadata(submodule)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
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
        let mut report = ScanReport::default();
        for entry in fs::read_dir(&root).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                let item_id = entry.file_name().to_string_lossy().into_owned();
                scan_workshop_item(&entry.path(), &item_id, &mut by_id, &mut report).unwrap();
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
        assert_eq!(modules[0].workshop_item_id.as_deref(), Some("1234567890"));
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
    fn detects_case_insensitive_submodule_filename() {
        let root = std::env::temp_dir().join(format!(
            "mb2-scanner-test-case-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root.join("999")).unwrap();
        fs::write(root.join("999").join("submodule.xml"), SAMPLE_SUBMODULE).unwrap();

        let mut by_id = HashMap::new();
        let mut report = ScanReport::default();
        scan_workshop_item(&root.join("999"), "999", &mut by_id, &mut report).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(by_id.len(), 1);
        assert!(by_id.contains_key("Test.Mod"));
    }

    #[test]
    fn prefers_game_install_for_official_duplicate() {
        let mut by_id = HashMap::new();
        insert_module(
            &mut by_id,
            InstalledModule {
                info: SubModuleInfo {
                    id: "Native".into(),
                    name: "Native".into(),
                    version: None,
                    singleplayer: true,
                    multiplayer: false,
                    depended_modules: Vec::new(),
                    depended_module_metadatas: Vec::new(),
                    modules_to_load_after_this: Vec::new(),
                    modules_to_load_before_this: Vec::new(),
                    dll_names: Vec::new(),
                    url: None,
                    folder_name: "Native".into(),
                },
                path: "/workshop/content/261550/1/Native".into(),
                source: ModuleSource::Workshop,
                workshop_item_id: Some("1".into()),
            },
        );
        insert_module(
            &mut by_id,
            InstalledModule {
                info: SubModuleInfo {
                    id: "Native".into(),
                    name: "Native".into(),
                    version: None,
                    singleplayer: true,
                    multiplayer: false,
                    depended_modules: Vec::new(),
                    depended_module_metadatas: Vec::new(),
                    modules_to_load_after_this: Vec::new(),
                    modules_to_load_before_this: Vec::new(),
                    dll_names: Vec::new(),
                    url: None,
                    folder_name: "Native".into(),
                },
                path: "/game/Modules/Native".into(),
                source: ModuleSource::Game,
                workshop_item_id: None,
            },
        );

        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id["Native"].source, ModuleSource::Game);
    }

    #[test]
    fn prefers_workshop_for_community_duplicate() {
        let mut by_id = HashMap::new();
        insert_module(
            &mut by_id,
            InstalledModule {
                info: SubModuleInfo {
                    id: "Bannerlord.Harmony".into(),
                    name: "Harmony".into(),
                    version: None,
                    singleplayer: true,
                    multiplayer: false,
                    depended_modules: Vec::new(),
                    depended_module_metadatas: Vec::new(),
                    modules_to_load_after_this: Vec::new(),
                    modules_to_load_before_this: Vec::new(),
                    dll_names: Vec::new(),
                    url: None,
                    folder_name: "Bannerlord.Harmony".into(),
                },
                path: "/game/Modules/Bannerlord.Harmony".into(),
                source: ModuleSource::Game,
                workshop_item_id: None,
            },
        );
        insert_module(
            &mut by_id,
            InstalledModule {
                info: SubModuleInfo {
                    id: "Bannerlord.Harmony".into(),
                    name: "Harmony".into(),
                    version: None,
                    singleplayer: true,
                    multiplayer: false,
                    depended_modules: Vec::new(),
                    depended_module_metadatas: Vec::new(),
                    modules_to_load_after_this: Vec::new(),
                    modules_to_load_before_this: Vec::new(),
                    dll_names: Vec::new(),
                    url: None,
                    folder_name: "Bannerlord.Harmony".into(),
                },
                path: "/steamapps/workshop/content/261550/2859188632".into(),
                source: ModuleSource::Workshop,
                workshop_item_id: Some("2859188632".into()),
            },
        );

        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id["Bannerlord.Harmony"].source, ModuleSource::Workshop);
    }

    #[test]
    #[ignore]
    fn workshop_scan_audit() {
        let paths = GamePaths::detect().unwrap();
        let (modules, report) = scan_modules_with_report(&paths).unwrap();
        eprintln!("report={report:?}");
        eprintln!(
            "total={} game={} workshop={}",
            modules.len(),
            report.modules_game,
            report.modules_workshop
        );
        for module in modules
            .iter()
            .filter(|m| m.source == ModuleSource::Workshop)
        {
            eprintln!(
                "  WS {} item={:?}",
                module.info.id, module.workshop_item_id
            );
        }
    }
}
