use crate::error::{Mb2Error, Result};
use crate::load_order::LoadOrderEntry;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::fs;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LauncherData {
    pub singleplayer: Vec<LoadOrderEntry>,
    pub dll_check_data: Vec<DllCheckEntry>,
}

#[derive(Debug, Clone)]
pub struct DllCheckEntry {
    pub id: String,
    pub dll_name: String,
    pub unknown: i32,
    pub is_selected: bool,
}

pub fn read_launcher_data(path: &Path) -> Result<LauncherData> {
    if !path.exists() {
        return Ok(LauncherData {
            singleplayer: Vec::new(),
            dll_check_data: Vec::new(),
        });
    }

    let content = fs::read_to_string(path)?;
    parse_launcher_data_xml(&content)
}

pub fn write_launcher_data(path: &Path, data: &LauncherData) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let xml = serialize_launcher_data_xml(data)?;
    fs::write(path, xml)?;
    Ok(())
}

fn parse_launcher_data_xml(content: &str) -> Result<LauncherData> {
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut singleplayer = Vec::new();
    let mut dll_check_data = Vec::new();

    let mut section = Section::None;
    let mut current_tag = String::new();

    let mut mod_id = String::new();
    let mut mod_selected = false;

    let mut dll_id = String::new();
    let mut dll_name = String::new();
    let mut dll_unknown = 0;
    let mut dll_selected = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match current_tag.as_str() {
                    "Singleplayer" => section = Section::Singleplayer,
                    "DLLCheckData" => section = Section::DllCheck,
                    "UserModData" => {
                        mod_id.clear();
                        mod_selected = false;
                    }
                    "DLLCheckDataEntry" => {
                        dll_id.clear();
                        dll_name.clear();
                        dll_unknown = 0;
                        dll_selected = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().into_owned();
                match (section, current_tag.as_str()) {
                    (Section::Singleplayer, "Id") => mod_id = text,
                    (Section::Singleplayer, "IsSelected") => mod_selected = text == "true",
                    (Section::DllCheck, "Id") => dll_id = text,
                    (Section::DllCheck, "DLLName") => dll_name = text,
                    (Section::DllCheck, "Unknown") => dll_unknown = text.parse().unwrap_or(0),
                    (Section::DllCheck, "IsSelected") => dll_selected = text == "true",
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"UserModData" if section == Section::Singleplayer && !mod_id.is_empty() => {
                        singleplayer.push(LoadOrderEntry {
                            module_id: std::mem::take(&mut mod_id),
                            enabled: mod_selected,
                        });
                        mod_selected = false;
                    }
                    b"DLLCheckDataEntry" if section == Section::DllCheck && !dll_id.is_empty() => {
                        dll_check_data.push(DllCheckEntry {
                            id: std::mem::take(&mut dll_id),
                            dll_name: std::mem::take(&mut dll_name),
                            unknown: dll_unknown,
                            is_selected: dll_selected,
                        });
                        dll_unknown = 0;
                        dll_selected = false;
                    }
                    b"Singleplayer" => section = Section::None,
                    b"DLLCheckData" => section = Section::None,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Mb2Error::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(LauncherData {
        singleplayer,
        dll_check_data,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Singleplayer,
    DllCheck,
}

fn serialize_launcher_data_xml(data: &LauncherData) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    writer.write_event(Event::Start(BytesStart::new("UserData")))?;

    writer.write_event(Event::Start(BytesStart::new("Singleplayer")))?;
    for entry in &data.singleplayer {
        writer.write_event(Event::Start(BytesStart::new("UserModData")))?;
        write_text_element(&mut writer, "Id", &entry.module_id)?;
        write_text_element(
            &mut writer,
            "IsSelected",
            if entry.enabled { "true" } else { "false" },
        )?;
        writer.write_event(Event::End(BytesEnd::new("UserModData")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("Singleplayer")))?;

    if !data.dll_check_data.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("DLLCheckData")))?;
        for entry in &data.dll_check_data {
            writer.write_event(Event::Start(BytesStart::new("DLLCheckDataEntry")))?;
            write_text_element(&mut writer, "Id", &entry.id)?;
            write_text_element(&mut writer, "DLLName", &entry.dll_name)?;
            write_text_element(&mut writer, "Unknown", &entry.unknown.to_string())?;
            write_text_element(
                &mut writer,
                "IsSelected",
                if entry.is_selected { "true" } else { "false" },
            )?;
            writer.write_event(Event::End(BytesEnd::new("DLLCheckDataEntry")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("DLLCheckData")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("UserData")))?;

    let bytes = writer.into_inner().into_inner();
    Ok(String::from_utf8(bytes).map_err(|e| Mb2Error::Xml(e.to_string()))?)
}

fn write_text_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    tag: &str,
    value: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_launcher_data() {
        let data = LauncherData {
            singleplayer: vec![
                LoadOrderEntry {
                    module_id: "Native".into(),
                    enabled: true,
                },
                LoadOrderEntry {
                    module_id: "Bannerlord.Harmony".into(),
                    enabled: true,
                },
            ],
            dll_check_data: vec![],
        };

        let xml = serialize_launcher_data_xml(&data).unwrap();
        let parsed = parse_launcher_data_xml(&xml).unwrap();
        assert_eq!(parsed.singleplayer.len(), 2);
        assert_eq!(parsed.singleplayer[0].module_id, "Native");
    }
}
