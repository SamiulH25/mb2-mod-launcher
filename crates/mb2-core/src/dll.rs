use crate::error::Result;
use crate::submodule::SubModuleInfo;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize)]
pub struct UnblockResult {
    pub files_processed: usize,
    pub files_unblocked: usize,
    pub errors: Vec<String>,
}

pub fn unblock_module_dlls(module_path: &Path, info: &SubModuleInfo) -> Result<UnblockResult> {
    let mut result = UnblockResult {
        files_processed: 0,
        files_unblocked: 0,
        errors: Vec::new(),
    };

    let bin_dirs = [
        module_path.join("bin/Win64_Shipping_Client"),
        module_path.join("bin/Gaming.Desktop.x64_Shipping_Client"),
    ];

    for dll_name in &info.dll_names {
        for bin_dir in &bin_dirs {
            let dll_path = bin_dir.join(dll_name);
            if dll_path.exists() {
                match unblock_file(&dll_path) {
                    Ok(changed) => {
                        result.files_processed += 1;
                        if changed {
                            result.files_unblocked += 1;
                        }
                    }
                    Err(err) => result.errors.push(format!("{}: {err}", dll_path.display())),
                }
            }
        }
    }

    if result.files_processed == 0 {
        unblock_all_dlls_recursive(module_path, &mut result)?;
    }

    Ok(result)
}

fn unblock_all_dlls_recursive(dir: &Path, result: &mut UnblockResult) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            unblock_all_dlls_recursive(&path, result)?;
        } else if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dll")) {
            match unblock_file(&path) {
                Ok(changed) => {
                    result.files_processed += 1;
                    if changed {
                        result.files_unblocked += 1;
                    }
                }
                Err(err) => result.errors.push(format!("{}: {err}", path.display())),
            }
        }
    }

    Ok(())
}

fn unblock_file(path: &Path) -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("xattr")
            .args(["-l", &path.to_string_lossy()])
            .output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let attrs = String::from_utf8_lossy(&output.stdout);
        if attrs.contains("user.zipro") || attrs.contains("user.dos") {
            let status = Command::new("xattr")
                .args(["-d", "user.zipro", &path.to_string_lossy()])
                .status();
            let _ = status;

            let status = Command::new("xattr")
                .args(["-d", "user.dos", &path.to_string_lossy()])
                .status();

            return Ok(status.map(|s| s.success()).unwrap_or(false));
        }

        Ok(false)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        Ok(false)
    }
}
