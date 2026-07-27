use crate::error::{Mb2Error, Result};
use crate::submodule::SubModuleInfo;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Official vanilla modules in launcher default order.
pub const OFFICIAL_MODULE_ORDER: &[&str] = &[
    "Native",
    "SandBoxCore",
    "CustomBattle",
    "Sandbox",
    "StoryMode",
    "BirthAndDeath",
];

/// Community framework mods that should stay at the top (after sorting deps).
pub const FRAMEWORK_PIN_ORDER: &[&str] = &[
    "Bannerlord.Harmony",
    "Bannerlord.ButterLib",
    "Bannerlord.UIExtenderEx",
    "Bannerlord.MBOptionScreen",
];

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
    let mut warnings = Vec::new();

    for module in &enabled_modules {
        for dep in &module.depended_modules {
            if dep.optional {
                continue;
            }
            if !enabled_ids.contains(dep.id.as_str()) && !is_official_module(&dep.id) {
                warnings.push(format!(
                    "'{}' requires '{}' which is not enabled",
                    module.name, dep.id
                ));
            }
        }
    }

    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    for module in &enabled_modules {
        graph.entry(module.id.clone()).or_default();
    }

    for module in &enabled_modules {
        for dep in &module.depended_modules {
            if enabled_ids.contains(dep.id.as_str()) {
                graph
                    .entry(dep.id.clone())
                    .or_default()
                    .insert(module.id.clone());
            }
        }
        for meta in &module.depended_module_metadatas {
            if !enabled_ids.contains(meta.id.as_str()) {
                continue;
            }
            match meta.order.as_deref() {
                Some("LoadBeforeThis") => {
                    graph
                        .entry(meta.id.clone())
                        .or_default()
                        .insert(module.id.clone());
                }
                Some("LoadAfterThis") => {
                    graph
                        .entry(meta.id.clone())
                        .or_default()
                        .insert(module.id.clone());
                }
                _ => {}
            }
        }
    }

    let sorted_ids = topological_sort(&graph)?;

    let mut final_order = apply_pin_order(&sorted_ids);

    for official in OFFICIAL_MODULE_ORDER {
        if enabled_ids.contains(*official) && !final_order.iter().any(|id| id == official) {
            final_order.push(official.to_string());
        }
    }

    for module in modules {
        if *enabled.get(&module.id).unwrap_or(&false)
            && !final_order.iter().any(|id| id == &module.id)
        {
            final_order.push(module.id.clone());
        }
    }

    final_order = dedupe_preserve_order(final_order);

    let order = final_order
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
        .collect();

    Ok(SortResult { order, warnings })
}

fn topological_sort(graph: &HashMap<String, HashSet<String>>) -> Result<Vec<String>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for node in graph.keys() {
        in_degree.entry(node.as_str()).or_insert(0);
    }
    for dependents in graph.values() {
        for dep in dependents {
            *in_degree.entry(dep.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    queue.make_contiguous().sort_unstable();

    let mut result = Vec::new();

    while let Some(node) = queue.pop_front() {
        result.push(node.to_string());
        if let Some(dependents) = graph.get(node) {
            let mut next_ready = Vec::new();
            for dependent in dependents {
                let entry = in_degree.get_mut(dependent.as_str()).unwrap();
                *entry -= 1;
                if *entry == 0 {
                    next_ready.push(dependent.as_str());
                }
            }
            next_ready.sort_unstable();
            for dep in next_ready {
                queue.push_back(dep);
            }
        }
    }

    if result.len() != graph.len() {
        return Err(Mb2Error::LoadOrder(
            "Circular dependency detected in mod load order".into(),
        ));
    }

    Ok(result)
}

fn apply_pin_order(sorted: &[String]) -> Vec<String> {
    let mut pinned: Vec<String> = FRAMEWORK_PIN_ORDER
        .iter()
        .filter(|id| sorted.iter().any(|s| s == *id))
        .map(|s| (*s).to_string())
        .collect();

    let pinned_set: HashSet<&str> = FRAMEWORK_PIN_ORDER.iter().copied().collect();
    let rest: Vec<String> = sorted
        .iter()
        .filter(|id| !pinned_set.contains(id.as_str()))
        .cloned()
        .collect();

    pinned.extend(rest);
    pinned
}

fn dedupe_preserve_order(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

pub fn is_official_module(id: &str) -> bool {
    OFFICIAL_MODULE_ORDER.contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::submodule::{DependedModule, DependedModuleMetadata};

    fn mod_info(id: &str, deps: Vec<&str>) -> SubModuleInfo {
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
            depended_module_metadatas: vec![DependedModuleMetadata {
                id: "Native".into(),
                order: Some("LoadBeforeThis".into()),
                version: None,
                optional: false,
            }],
            dll_names: vec![],
            url: None,
            folder_name: id.into(),
        }
    }

    #[test]
    fn dependency_sorts_after_requirement() {
        let modules = vec![
            mod_info("Bannerlord.Harmony", vec![]),
            mod_info("MyMod", vec!["Bannerlord.Harmony"]),
        ];
        let mut enabled = HashMap::new();
        enabled.insert("Bannerlord.Harmony".into(), true);
        enabled.insert("MyMod".into(), true);

        let result = auto_sort(&modules, &enabled).unwrap();
        let enabled_order: Vec<_> = result
            .order
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.module_id.as_str())
            .collect();

        let harmony_pos = enabled_order
            .iter()
            .position(|&id| id == "Bannerlord.Harmony")
            .unwrap();
        let mymod_pos = enabled_order.iter().position(|&id| id == "MyMod").unwrap();
        assert!(harmony_pos < mymod_pos);
    }
}
