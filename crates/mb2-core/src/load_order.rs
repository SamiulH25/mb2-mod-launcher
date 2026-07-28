//! Load-order sorting aligned with [BUTR Bannerlord.ModuleManager](https://github.com/BUTR/Bannerlord.ModuleManager).
//!
//! Bannerlord resolves sort dependencies from:
//! - `<DependedModules>` and `<DependedModuleMetadatas order="LoadBeforeThis">`
//! - `<ModulesToLoadAfterThis>` / `<DependedModuleMetadatas order="LoadAfterThis">` on *other* modules
//! - Official modules keep their vanilla relative order via dependency declarations

use crate::error::{Mb2Error, Result};
use crate::submodule::SubModuleInfo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Framework mods that must stay above vanilla when enabled (display + validation helpers).
pub const FRAMEWORK_PIN_ORDER: &[&str] = &[
    "Bannerlord.Harmony",
    "BetterExceptionWindow",
    "Bannerlord.ButterLib",
    "Bannerlord.UIExtenderEx",
    "Bannerlord.MBOptionScreen",
];

/// Vanilla module order used by the official launcher (top → bottom).
pub const OFFICIAL_MODULE_ORDER: &[&str] = &[
    "Native",
    "SandBoxCore",
    "CustomBattle",
    "Sandbox",
    "StoryMode",
    "BirthAndDeath",
    "FastMode",
    "NavalDLC",
    "Multiplayer",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadRelation {
    Before,
    After,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOrderEntry {
    pub module_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortResult {
    pub order: Vec<LoadOrderEntry>,
    pub warnings: Vec<String>,
}

pub fn auto_sort(
    modules: &[SubModuleInfo],
    enabled: &HashMap<String, bool>,
) -> Result<SortResult> {
    let enabled_modules: Vec<&SubModuleInfo> = modules
        .iter()
        .filter(|m| *enabled.get(&m.id).unwrap_or(&false))
        .collect();

    let enabled_ids: HashSet<&str> = enabled_modules.iter().map(|m| m.id.as_str()).collect();
    let mut warnings = collect_dependency_warnings(modules, &enabled_modules, &enabled_ids);

    let sorted_ids = butr_sort_enabled(modules, &enabled_modules, &enabled_ids)?;
    let mut sorted_ids = ensure_community_after_officials(&sorted_ids, &enabled_ids);

    if let Some(violations) = validate_sorted_order(modules, &sorted_ids, &enabled_ids) {
        warnings.extend(violations);
        repair_dependency_violations(&mut sorted_ids, modules, &enabled_ids);
        warnings.extend(validate_sorted_order(modules, &sorted_ids, &enabled_ids).unwrap_or_default());
    }

    let order = sorted_ids
        .into_iter()
        .map(|module_id| LoadOrderEntry {
            enabled: true,
            module_id,
        })
        .chain(
            modules
                .iter()
                .filter(|m| !*enabled.get(&m.id).unwrap_or(&false))
                .map(|m| LoadOrderEntry {
                    module_id: m.id.clone(),
                    enabled: false,
                }),
        )
        .collect::<Vec<LoadOrderEntry>>();

    Ok(SortResult { order, warnings })
}

/// DFS topological sort matching `Bannerlord.ModuleManager.ModuleSorter`.
fn butr_sort_enabled(
    all_modules: &[SubModuleInfo],
    enabled_modules: &[&SubModuleInfo],
    enabled_ids: &HashSet<&str>,
) -> Result<Vec<String>> {
    let mut sort_input: Vec<&SubModuleInfo> = enabled_modules.to_vec();
    sort_input.sort_by(|a, b| module_sort_key(a).cmp(&module_sort_key(b)));

    let mut result: Vec<&SubModuleInfo> = Vec::new();
    let mut visited: HashSet<&str> = HashSet::new();

    for module in sort_input {
        visit_for_sort(module, all_modules, enabled_ids, &mut result, &mut visited);
    }

    if result.len() != enabled_modules.len() {
        return Err(Mb2Error::LoadOrder(
            "Could not produce a complete load order — check for circular mod dependencies"
                .into(),
        ));
    }

    Ok(result.iter().map(|m| m.id.clone()).collect())
}

fn visit_for_sort<'a>(
    module: &'a SubModuleInfo,
    all_modules: &'a [SubModuleInfo],
    enabled_ids: &HashSet<&str>,
    result: &mut Vec<&'a SubModuleInfo>,
    visited: &mut HashSet<&'a str>,
) {
    if visited.contains(module.id.as_str()) {
        return;
    }
    visited.insert(module.id.as_str());

    for dep in sort_dependencies(all_modules, module, enabled_ids) {
        visit_for_sort(dep, all_modules, enabled_ids, result, visited);
    }

    result.push(module);
}

/// Modules that must appear *before* `module` in the load order.
fn sort_dependencies<'a>(
    all_modules: &'a [SubModuleInfo],
    module: &'a SubModuleInfo,
    enabled_ids: &HashSet<&str>,
) -> Vec<&'a SubModuleInfo> {
    let mut deps = Vec::new();

    for dep_id in dependencies_load_before_this(module) {
        if let Some(dep) = find_enabled_module(all_modules, &dep_id, enabled_ids) {
            deps.push(dep);
        }
    }

    for other in all_modules {
        if !enabled_ids.contains(other.id.as_str()) {
            continue;
        }
        for dep_id in dependencies_load_after_this(other) {
            if dep_id == module.id {
                deps.push(other);
                break;
            }
        }
    }

    deps.sort_by_key(|m| m.id.as_str());
    deps.dedup_by_key(|m| m.id.as_str());
    deps
}

fn dependencies_load_before_this(module: &SubModuleInfo) -> Vec<String> {
    let mut ids = Vec::new();

    for meta in &module.depended_module_metadatas {
        if meta.order.as_deref() == Some("LoadBeforeThis") {
            ids.push(meta.id.clone());
        }
    }

    for dep in &module.depended_modules {
        let overridden_by_after = module.depended_module_metadatas.iter().any(|meta| {
            meta.id == dep.id && meta.order.as_deref() == Some("LoadAfterThis")
        });
        if !overridden_by_after {
            ids.push(dep.id.clone());
        }
    }

    ids.sort();
    ids.dedup();
    ids
}

fn dependencies_load_after_this(module: &SubModuleInfo) -> Vec<String> {
    let mut ids: Vec<String> = module
        .modules_to_load_after_this
        .iter()
        .cloned()
        .collect();

    for meta in &module.depended_module_metadatas {
        if meta.order.as_deref() == Some("LoadAfterThis") {
            ids.push(meta.id.clone());
        }
    }

    ids.sort();
    ids.dedup();
    ids
}

fn dependencies_to_validate(module: &SubModuleInfo) -> Vec<(String, LoadRelation)> {
    let mut rules = Vec::new();

    for id in dependencies_load_before_this(module) {
        rules.push((id, LoadRelation::Before));
    }
    for id in dependencies_load_after_this(module) {
        rules.push((id, LoadRelation::After));
    }

    rules.sort_by(|a, b| a.0.cmp(&b.0));
    rules.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    rules
}

fn validate_sorted_order(
    all_modules: &[SubModuleInfo],
    order: &[String],
    enabled_ids: &HashSet<&str>,
) -> Option<Vec<String>> {
    let mut warnings = Vec::new();
    let positions: HashMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    for module in all_modules {
        if !enabled_ids.contains(module.id.as_str()) {
            continue;
        }
        let Some(&module_pos) = positions.get(module.id.as_str()) else {
            continue;
        };

        for (dep_id, relation) in dependencies_to_validate(module) {
            if !enabled_ids.contains(dep_id.as_str()) {
                continue;
            }
            let Some(&dep_pos) = positions.get(dep_id.as_str()) else {
                continue;
            };

            let violated = match relation {
                LoadRelation::Before => dep_pos > module_pos,
                LoadRelation::After => dep_pos < module_pos,
            };

            if violated {
                let msg = match relation {
                    LoadRelation::Before => format!(
                        "'{}' must load after '{}' (dependency conflict)",
                        module.id, dep_id
                    ),
                    LoadRelation::After => format!(
                        "'{}' must load before '{}' (dependency conflict)",
                        module.id, dep_id
                    ),
                };
                warnings.push(msg);
            }
        }
    }

    if warnings.is_empty() {
        None
    } else {
        Some(warnings)
    }
}

fn collect_dependency_warnings(
    all_modules: &[SubModuleInfo],
    enabled_modules: &[&SubModuleInfo],
    enabled_ids: &HashSet<&str>,
) -> Vec<String> {
    let mut warnings = Vec::new();

    for module in enabled_modules {
        for (dep_id, relation) in dependencies_to_validate(module) {
            if relation != LoadRelation::Before {
                continue;
            }

            let optional = module
                .depended_modules
                .iter()
                .find(|d| d.id == dep_id)
                .map(|d| d.optional)
                .unwrap_or_else(|| {
                    module
                        .depended_module_metadatas
                        .iter()
                        .find(|m| m.id == dep_id)
                        .map(|m| m.optional)
                        .unwrap_or(false)
                });

            if optional {
                continue;
            }

            if !enabled_ids.contains(dep_id.as_str()) && !is_official_module(&dep_id) {
                warnings.push(format!(
                    "'{}' requires '{}' which is not enabled",
                    module.name, dep_id
                ));
                continue;
            }

            if let Some(required) = required_version_for(module, &dep_id) {
                if let Some(provider) = all_modules.iter().find(|m| m.id == dep_id) {
                    if let Some(installed) = provider.version.as_deref() {
                        if !version_at_least(installed, &required) {
                            warnings.push(format!(
                                "'{}' requires '{}' >= {} but installed version is {}",
                                module.name, dep_id, required, installed
                            ));
                        }
                    }
                }
            }
        }
    }

    warnings
}

fn required_version_for(module: &SubModuleInfo, dep_id: &str) -> Option<String> {
    module
        .depended_modules
        .iter()
        .find(|d| d.id == dep_id)
        .and_then(|d| d.version.clone())
        .or_else(|| {
            module
                .depended_module_metadatas
                .iter()
                .find(|m| m.id == dep_id)
                .and_then(|m| m.version.clone())
        })
}

fn ensure_community_after_officials(order: &[String], _enabled_ids: &HashSet<&str>) -> Vec<String> {
    let mut officials = Vec::new();
    let mut community = Vec::new();

    for id in order {
        if is_official_module(id) || is_framework_module(id) {
            officials.push(id.clone());
        } else {
            community.push(id.clone());
        }
    }

    officials.extend(community);
    officials
}

fn repair_dependency_violations(
    order: &mut Vec<String>,
    all_modules: &[SubModuleInfo],
    enabled_ids: &HashSet<&str>,
) {
    let module_map: HashMap<&str, &SubModuleInfo> = all_modules
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect();

    let max_passes = order.len().saturating_mul(order.len()).max(1);
    for _ in 0..max_passes {
        let mut changed = false;
        for module_id in order.clone() {
            let Some(module) = module_map.get(module_id.as_str()) else {
                continue;
            };
            let Some(module_pos) = order.iter().position(|id| id == &module.id) else {
                continue;
            };

            let mut target_pos = module_pos;
            for (dep_id, relation) in dependencies_to_validate(module) {
                if !enabled_ids.contains(dep_id.as_str()) {
                    continue;
                }
                let Some(dep_pos) = order.iter().position(|id| id == &dep_id) else {
                    continue;
                };
                match relation {
                    LoadRelation::Before if dep_pos > module_pos => {
                        target_pos = target_pos.max(dep_pos);
                    }
                    LoadRelation::After if dep_pos < module_pos => {
                        target_pos = target_pos.max(dep_pos);
                    }
                    _ => {}
                }
            }

            if target_pos > module_pos {
                order.retain(|id| id != &module.id);
                order.insert(target_pos.min(order.len()), module.id.clone());
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
}

fn find_enabled_module<'a>(
    modules: &'a [SubModuleInfo],
    id: &str,
    enabled_ids: &HashSet<&str>,
) -> Option<&'a SubModuleInfo> {
    if !enabled_ids.contains(id) {
        return None;
    }
    modules.iter().find(|m| m.id == id)
}

fn module_sort_key(module: &SubModuleInfo) -> (u8, u32, &str) {
    if is_official_module(&module.id) {
        let index = OFFICIAL_MODULE_ORDER
            .iter()
            .position(|id| *id == module.id)
            .unwrap_or(OFFICIAL_MODULE_ORDER.len()) as u32;
        return (0, index, module.id.as_str());
    }
    if is_framework_module(&module.id) {
        let index = FRAMEWORK_PIN_ORDER
            .iter()
            .position(|id| *id == module.id)
            .unwrap_or(FRAMEWORK_PIN_ORDER.len()) as u32;
        return (1, index, module.id.as_str());
    }
    (2, 0, module.id.as_str())
}

pub fn parse_bannerlord_version(version: &str) -> Option<Vec<u32>> {
    let trimmed = version.trim().strip_prefix('v').unwrap_or(version.trim());
    let parts: Vec<u32> = trimmed
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts)
    }
}

pub fn version_at_least(installed: &str, required: &str) -> bool {
    match (
        parse_bannerlord_version(installed),
        parse_bannerlord_version(required),
    ) {
        (Some(installed_parts), Some(required_parts)) => {
            for i in 0..required_parts.len().max(installed_parts.len()) {
                let installed_part = installed_parts.get(i).copied().unwrap_or(0);
                let required_part = required_parts.get(i).copied().unwrap_or(0);
                if installed_part != required_part {
                    return installed_part > required_part;
                }
            }
            true
        }
        _ => true,
    }
}

pub fn is_official_module(id: &str) -> bool {
    OFFICIAL_MODULE_ORDER.contains(&id)
}

pub fn is_framework_module(id: &str) -> bool {
    FRAMEWORK_PIN_ORDER.contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::submodule::{DependedModule, DependedModuleMetadata};

    fn mod_info(
        id: &str,
        deps: Vec<&str>,
        metas: Vec<(&str, &str)>,
    ) -> SubModuleInfo {
        SubModuleInfo {
            id: id.into(),
            name: id.into(),
            version: None,
            singleplayer: true,
            multiplayer: false,
            depended_modules: deps
                .into_iter()
                .map(|d| DependedModule {
                    id: d.into(),
                    version: None,
                    optional: false,
                })
                .collect(),
            depended_module_metadatas: metas
                .into_iter()
                .map(|(dep_id, order)| DependedModuleMetadata {
                    id: dep_id.into(),
                    order: Some(order.into()),
                    version: None,
                    optional: false,
                })
                .collect(),
            modules_to_load_after_this: Vec::new(),
            modules_to_load_before_this: Vec::new(),
            dll_names: vec![],
            url: None,
            folder_name: id.into(),
        }
    }

    fn enabled_order(result: &SortResult) -> Vec<String> {
        result
            .order
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.module_id.clone())
            .collect()
    }

    fn pos(order: &[String], id: &str) -> usize {
        order.iter().position(|m| m == id).unwrap()
    }

    #[test]
    fn framework_stack_precedes_official_modules() {
        let mut harmony = mod_info("Bannerlord.Harmony", vec![], vec![]);
        harmony.modules_to_load_after_this = vec![
            "Native".into(),
            "SandBoxCore".into(),
            "Sandbox".into(),
            "StoryMode".into(),
            "CustomBattle".into(),
        ];

        let mut butter = mod_info("Bannerlord.ButterLib", vec!["Bannerlord.Harmony"], vec![]);
        butter.modules_to_load_after_this = harmony.modules_to_load_after_this.clone();

        let modules = vec![
            harmony,
            butter,
            {
                let mut ui = mod_info("Bannerlord.UIExtenderEx", vec!["Bannerlord.ButterLib"], vec![]);
                ui.modules_to_load_after_this = vec!["Native".into()];
                ui
            },
            {
                let mut mcm = mod_info(
                    "Bannerlord.MBOptionScreen",
                    vec!["Bannerlord.UIExtenderEx"],
                    vec![],
                );
                mcm.modules_to_load_after_this = vec!["Native".into()];
                mcm
            },
            mod_info("Native", vec![], vec![]),
            mod_info("SandBoxCore", vec!["Native"], vec![]),
            mod_info("Sandbox", vec!["Native", "SandBoxCore"], vec![]),
        ];
        let mut enabled = HashMap::new();
        for id in [
            "Bannerlord.Harmony",
            "Bannerlord.ButterLib",
            "Bannerlord.UIExtenderEx",
            "Bannerlord.MBOptionScreen",
            "Native",
            "SandBoxCore",
            "Sandbox",
        ] {
            enabled.insert(id.into(), true);
        }

        let order = enabled_order(&auto_sort(&modules, &enabled).unwrap());

        assert!(pos(&order, "Bannerlord.Harmony") < pos(&order, "Native"));
        assert!(pos(&order, "Bannerlord.ButterLib") < pos(&order, "Native"));
        assert!(pos(&order, "Bannerlord.UIExtenderEx") < pos(&order, "Native"));
        assert!(pos(&order, "Bannerlord.MBOptionScreen") < pos(&order, "Native"));
        assert!(pos(&order, "Bannerlord.Harmony") < pos(&order, "Bannerlord.ButterLib"));
        assert!(pos(&order, "Native") < pos(&order, "SandBoxCore"));
        assert!(pos(&order, "SandBoxCore") < pos(&order, "Sandbox"));
    }

    #[test]
    fn community_mod_loads_after_framework_and_officials() {
        let mut harmony = mod_info("Bannerlord.Harmony", vec![], vec![]);
        harmony.modules_to_load_after_this = vec!["Native".into()];

        let modules = vec![
            harmony,
            mod_info("Native", vec![], vec![]),
            mod_info("SandBoxCore", vec!["Native"], vec![]),
            mod_info("Sandbox", vec!["Native", "SandBoxCore"], vec![]),
            mod_info("MyMod", vec!["Bannerlord.Harmony", "Native", "Sandbox"], vec![]),
        ];
        let mut enabled = HashMap::new();
        for id in [
            "Bannerlord.Harmony",
            "Native",
            "SandBoxCore",
            "Sandbox",
            "MyMod",
        ] {
            enabled.insert(id.into(), true);
        }

        let order = enabled_order(&auto_sort(&modules, &enabled).unwrap());
        assert!(pos(&order, "Bannerlord.Harmony") < pos(&order, "Native"));
        assert!(pos(&order, "Sandbox") < pos(&order, "MyMod"));
    }

    #[test]
    fn respects_load_before_this_metadata() {
        let mut harmony = mod_info("Bannerlord.Harmony", vec![], vec![]);
        harmony.modules_to_load_after_this = vec!["Native".into()];

        let modules = vec![
            harmony,
            mod_info(
                "MyMod",
                vec![],
                vec![("Bannerlord.Harmony", "LoadBeforeThis")],
            ),
            mod_info("Native", vec![], vec![]),
        ];
        let mut enabled = HashMap::new();
        for id in ["Native", "Bannerlord.Harmony", "MyMod"] {
            enabled.insert(id.into(), true);
        }

        let order = enabled_order(&auto_sort(&modules, &enabled).unwrap());
        assert!(pos(&order, "Bannerlord.Harmony") < pos(&order, "MyMod"));
    }

    #[test]
    fn butterlib_does_not_require_native_before_it() {
        let mut butter = mod_info("Bannerlord.ButterLib", vec!["Bannerlord.Harmony", "Native"], vec![]);
        butter.modules_to_load_after_this = vec![
            "Native".into(),
            "SandBoxCore".into(),
            "Sandbox".into(),
        ];
        butter.depended_module_metadatas = vec![
            DependedModuleMetadata {
                id: "Bannerlord.Harmony".into(),
                order: Some("LoadBeforeThis".into()),
                version: None,
                optional: false,
            },
            DependedModuleMetadata {
                id: "Native".into(),
                order: Some("LoadAfterThis".into()),
                version: None,
                optional: false,
            },
        ];

        let mut harmony = mod_info("Bannerlord.Harmony", vec![], vec![]);
        harmony.modules_to_load_after_this = vec!["Native".into(), "SandBoxCore".into()];

        let modules = vec![
            harmony,
            butter,
            mod_info("Native", vec![], vec![]),
            mod_info("SandBoxCore", vec!["Native"], vec![]),
            mod_info("Sandbox", vec!["Native", "SandBoxCore"], vec![]),
        ];
        let mut enabled = HashMap::new();
        for id in [
            "Bannerlord.Harmony",
            "Bannerlord.ButterLib",
            "Native",
            "SandBoxCore",
            "Sandbox",
        ] {
            enabled.insert(id.into(), true);
        }

        let order = enabled_order(&auto_sort(&modules, &enabled).unwrap());
        assert!(pos(&order, "Bannerlord.Harmony") < pos(&order, "Bannerlord.ButterLib"));
        assert!(pos(&order, "Bannerlord.ButterLib") < pos(&order, "Native"));
    }

    #[test]
    fn community_mods_load_after_last_enabled_official() {
        let mut harmony = mod_info("Bannerlord.Harmony", vec![], vec![]);
        harmony.modules_to_load_after_this = vec!["Native".into(), "StoryMode".into()];

        let modules = vec![
            harmony,
            mod_info("Native", vec![], vec![]),
            mod_info("SandBoxCore", vec!["Native"], vec![]),
            mod_info("Sandbox", vec!["Native", "SandBoxCore"], vec![]),
            mod_info(
                "StoryMode",
                vec!["Native", "SandBoxCore", "Sandbox"],
                vec![],
            ),
            mod_info(
                "Fourberie",
                vec!["Native", "SandBoxCore", "Sandbox", "StoryMode"],
                vec![],
            ),
        ];
        let mut enabled = HashMap::new();
        for id in [
            "Bannerlord.Harmony",
            "Native",
            "SandBoxCore",
            "Sandbox",
            "StoryMode",
            "Fourberie",
        ] {
            enabled.insert(id.into(), true);
        }

        let order = enabled_order(&auto_sort(&modules, &enabled).unwrap());
        assert!(pos(&order, "StoryMode") < pos(&order, "Fourberie"));
        assert!(pos(&order, "Bannerlord.Harmony") < pos(&order, "Fourberie"));
    }

    #[test]
    fn version_at_least_compares_bannerlord_versions() {
        assert!(version_at_least("v2.4.2.0", "v2.4.2"));
        assert!(!version_at_least("v2.3.1", "v2.4.2"));
    }

    #[test]
    #[ignore = "requires local Bannerlord install"]
    fn real_install_sorts_without_cycles() {
        use crate::paths::GamePaths;
        use crate::scanner::scan_modules;

        let paths = GamePaths::detect().expect("game");
        let modules = scan_modules(&paths).expect("scan");
        let enabled: HashMap<String, bool> =
            modules.iter().map(|m| (m.info.id.clone(), true)).collect();
        let infos: Vec<_> = modules.iter().map(|m| m.info.clone()).collect();
        let result = auto_sort(&infos, &enabled).expect("sort");
        let order: Vec<_> = result
            .order
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.module_id.clone())
            .collect();

        let harmony = order.iter().position(|id| id == "Bannerlord.Harmony").unwrap();
        let native = order.iter().position(|id| id == "Native").unwrap();
        assert!(
            harmony < native,
            "Harmony must load before Native; got: {:?}",
            &order[..order.len().min(20)]
        );

        if let Some(butter) = order.iter().position(|id| id == "Bannerlord.ButterLib") {
            assert!(
                butter < native,
                "ButterLib must load before Native; got: {:?}",
                &order[..order.len().min(20)]
            );
        }

        let enabled_ids: HashSet<&str> = enabled.keys().map(|s| s.as_str()).collect();
        assert!(
            validate_sorted_order(&infos, &order, &enabled_ids).is_none(),
            "validation failed: {:?}",
            result.warnings
        );
    }
}
