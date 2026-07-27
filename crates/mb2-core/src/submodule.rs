use crate::error::{Mb2Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependedModule {
    pub id: String,
    pub version: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependedModuleMetadata {
    pub id: String,
    pub order: Option<String>,
    pub version: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubModuleInfo {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub singleplayer: bool,
    pub multiplayer: bool,
    pub depended_modules: Vec<DependedModule>,
    pub depended_module_metadatas: Vec<DependedModuleMetadata>,
    pub dll_names: Vec<String>,
    pub url: Option<String>,
    pub folder_name: String,
}

pub fn parse_submodule_xml(content: &str, folder_name: &str) -> Result<SubModuleInfo> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut id = None;
    let mut name = None;
    let mut version = None;
    let mut singleplayer = true;
    let mut multiplayer = false;
    let mut url = None;
    let mut depended_modules = Vec::new();
    let mut depended_module_metadatas = Vec::new();
    let mut dll_names = Vec::new();

    let mut in_depended_modules = false;
    let mut in_depended_module_metadatas = false;
    let mut in_submodules = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let current_element = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match current_element.as_str() {
                    "DependedModules" => in_depended_modules = true,
                    "DependedModuleMetadatas" => in_depended_module_metadatas = true,
                    "SubModules" => in_submodules = true,
                    "DependedModule" if in_depended_modules => {
                        let attrs = attribute_map(e);
                        if let Some(dep_id) = attrs.get("Id").cloned() {
                            depended_modules.push(DependedModule {
                                id: dep_id,
                                version: attrs.get("DependentVersion").cloned(),
                                optional: attrs
                                    .get("Optional")
                                    .map(|v| v == "true")
                                    .unwrap_or(false),
                            });
                        }
                    }
                    "DependedModuleMetadata" if in_depended_module_metadatas => {
                        let attrs = attribute_map(e);
                        if let Some(dep_id) = attrs.get("id").cloned() {
                            depended_module_metadatas.push(DependedModuleMetadata {
                                id: dep_id,
                                order: attrs.get("order").cloned(),
                                version: attrs.get("version").cloned(),
                                optional: attrs
                                    .get("optional")
                                    .map(|v| v == "true")
                                    .unwrap_or(false),
                            });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attrs = attribute_map(e);
                match tag.as_str() {
                    "Name" => name = attrs.get("value").cloned(),
                    "Id" => id = attrs.get("value").cloned(),
                    "Version" => version = attrs.get("value").cloned(),
                    "SingleplayerModule" => {
                        singleplayer = attrs.get("value").map(|v| v == "true").unwrap_or(true)
                    }
                    "MultiplayerModule" => {
                        multiplayer = attrs.get("value").map(|v| v == "true").unwrap_or(false)
                    }
                    "Url" => url = attrs.get("value").cloned(),
                    "DLLName" if in_submodules => {
                        if let Some(dll) = attrs.get("value").cloned() {
                            dll_names.push(dll);
                        }
                    }
                    "DependedModule" if in_depended_modules => {
                        if let Some(dep_id) = attrs.get("Id").cloned() {
                            depended_modules.push(DependedModule {
                                id: dep_id,
                                version: attrs.get("DependentVersion").cloned(),
                                optional: attrs
                                    .get("Optional")
                                    .map(|v| v == "true")
                                    .unwrap_or(false),
                            });
                        }
                    }
                    "DependedModuleMetadata" if in_depended_module_metadatas => {
                        if let Some(dep_id) = attrs.get("id").cloned() {
                            depended_module_metadatas.push(DependedModuleMetadata {
                                id: dep_id,
                                order: attrs.get("order").cloned(),
                                version: attrs.get("version").cloned(),
                                optional: attrs
                                    .get("optional")
                                    .map(|v| v == "true")
                                    .unwrap_or(false),
                            });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"DependedModules" => in_depended_modules = false,
                    b"DependedModuleMetadatas" => in_depended_module_metadatas = false,
                    b"SubModules" => in_submodules = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Mb2Error::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    let id = id.ok_or_else(|| Mb2Error::Xml("SubModule.xml missing Id".into()))?;
    let name = name.unwrap_or_else(|| id.clone());

    Ok(SubModuleInfo {
        id,
        name,
        version,
        singleplayer,
        multiplayer,
        depended_modules,
        depended_module_metadatas,
        dll_names,
        url,
        folder_name: folder_name.to_string(),
    })
}

pub fn parse_submodule_file(path: &Path) -> Result<SubModuleInfo> {
    let folder_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let content = std::fs::read_to_string(path)?;
    parse_submodule_xml(&content, &folder_name)
}

fn attribute_map(
    e: &quick_xml::events::BytesStart<'_>,
) -> HashMap<String, String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .map(|a| {
            let key = String::from_utf8_lossy(a.key.as_ref()).into_owned();
            let value = String::from_utf8_lossy(&a.value).into_owned();
            (key, value)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Module>
  <Name value="Harmony"/>
  <Id value="Bannerlord.Harmony"/>
  <Version value="v2.3.1"/>
  <SingleplayerModule value="true"/>
  <DependedModules>
    <DependedModule Id="Native"/>
  </DependedModules>
  <DependedModuleMetadatas>
    <DependedModuleMetadata id="Native" order="LoadBeforeThis"/>
  </DependedModuleMetadatas>
  <SubModules>
    <SubModule>
      <DLLName value="0Harmony.dll"/>
    </SubModule>
  </SubModules>
  <Url value="https://www.nexusmods.com/mountandblade2bannerlord/mods/2006"/>
</Module>"#;

    #[test]
    fn parses_sample_submodule() {
        let info = parse_submodule_xml(SAMPLE, "Bannerlord.Harmony").unwrap();
        assert_eq!(info.id, "Bannerlord.Harmony");
        assert_eq!(info.name, "Harmony");
        assert_eq!(info.depended_modules.len(), 1);
        assert_eq!(info.dll_names, vec!["0Harmony.dll"]);
        assert!(info.url.unwrap().contains("nexusmods.com"));
    }
}
