use crate::default_config_dir;
use crate::error::{Mb2Error, Result};
use crate::load_order::LoadOrderEntry;
use crate::paths::{GamePaths, STEAM_APP_ID};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const GAME_EXE_REL: &str = "bin/Win64_Shipping_Client/Bannerlord.exe";

/// Build launch args for a direct `Bannerlord.exe` start.
///
/// The TaleWorlds launcher reads `LauncherData.xml`; the game executable does not when
/// bypassing the launcher. Both must stay in sync: save XML first, then pass the same
/// enabled module list here via `_MODULES_*`.
pub fn build_module_launch_args(entries: &[LoadOrderEntry]) -> Vec<String> {
    let enabled_ids: Vec<&str> = entries
        .iter()
        .filter(|entry| entry.enabled)
        .map(|entry| entry.module_id.as_str())
        .collect();

    if enabled_ids.is_empty() {
        return vec!["/singleplayer".to_string()];
    }

    let modules = enabled_ids.join("*");
    vec![
        "/singleplayer".to_string(),
        format!("_MODULES_*{modules}*_MODULES_"),
    ]
}

/// Starts the game executable directly via Proton (not the TaleWorlds launcher).
/// Steam must be running so the overlay can hook in via LD_PRELOAD.
pub fn launch_bannerlord(paths: &GamePaths, entries: &[LoadOrderEntry]) -> Result<()> {
    let exe = paths.game_root.join(GAME_EXE_REL);
    if !exe.is_file() {
        return Err(Mb2Error::GameNotFound(format!(
            "Game executable not found: {}",
            exe.display()
        )));
    }

    let args = build_module_launch_args(entries);
    log_launch(&format!(
        "Launch requested with {} enabled module(s)",
        entries.iter().filter(|e| e.enabled).count()
    ));
    if let Some(modules_arg) = args.iter().find(|a| a.starts_with("_MODULES_")) {
        let preview: String = modules_arg.chars().take(120).collect();
        log_launch(&format!("  modules arg: {preview}..."));
    }

    let _ = ensure_steam_client_running()?;

    if launch_with_proton(paths, &exe, &args)? {
        log_launch("Started via Proton + game exe (Steam overlay preload enabled)");
        return Ok(());
    }

    log_launch("Proton launch failed; falling back to steam -applaunch (TaleWorlds launcher)");
    if launch_with_steam_applaunch()? {
        return Ok(());
    }

    log_launch("All launch methods failed");
    Err(Mb2Error::GameNotFound(
        "Could not launch Bannerlord. Start Steam, enable the in-game overlay, then try again. \
         Details: ~/.config/mb2-mod-launcher/launch.log"
            .into(),
    ))
}

fn launch_with_proton(paths: &GamePaths, exe: &Path, args: &[String]) -> Result<bool> {
    let compat_data = match resolve_compat_data(paths) {
        Some(path) => path,
        None => {
            log_launch("Proton launch skipped: no compatdata prefix found");
            return Ok(false);
        }
    };

    let steam_root = match find_steam_client_root() {
        Some(path) => path,
        None => {
            log_launch("Proton launch skipped: Steam install not found");
            return Ok(false);
        }
    };

    let proton = match find_proton_tool(&compat_data, &steam_root) {
        Some(path) => path,
        None => {
            log_launch("Proton launch skipped: no Proton tool found");
            return Ok(false);
        }
    };

    let proton_bin = proton.join("proton");
    if !proton_bin.is_file() {
        log_launch(&format!(
            "Proton launch skipped: missing {}",
            proton_bin.display()
        ));
        return Ok(false);
    }

    let work_dir = exe
        .parent()
        .ok_or_else(|| Mb2Error::GameNotFound("Invalid game executable path.".into()))?;

    write_steam_appid(work_dir)?;

    let mut command = Command::new(&proton_bin);
    command
        .current_dir(work_dir)
        .arg("waitforexitandrun")
        .arg(exe)
        .args(args)
        .env("STEAM_COMPAT_APP_ID", STEAM_APP_ID)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_root)
        .env("STEAM_COMPAT_DATA_PATH", &compat_data)
        .env("WINEDLLOVERRIDES", "winemenubuilder.exe=d")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    apply_steam_overlay_env(&mut command, &steam_root);

    log_launch(&format!(
        "Proton: {} waitforexitandrun {} {}",
        proton_bin.display(),
        exe.display(),
        args.join(" ")
    ));
    log_launch(&format!(
        "  STEAM_COMPAT_DATA_PATH={}",
        compat_data.display()
    ));

    match command.spawn() {
        Ok(_) => Ok(true),
        Err(error) => {
            log_launch(&format!("Proton spawn failed: {error}"));
            Ok(false)
        }
    }
}

/// Starts Bannerlord through Steam's default entry point (TaleWorlds launcher).
/// Used only as a fallback when direct Proton launch fails.
fn launch_with_steam_applaunch() -> Result<bool> {
    if !is_steam_client_running() {
        log_launch("Steam applaunch skipped: client not running");
        return Ok(false);
    }

    let command_args = ["-applaunch", STEAM_APP_ID];

    if let Some(steam) = find_steam_executable() {
        let steam_str = steam.to_string_lossy();
        log_launch(&format!("Steam: {steam_str} {}", command_args.join(" ")));
        if spawn_detached(&steam_str, &command_args) {
            return Ok(true);
        }
    }

    let flatpak_args = ["run", "com.valvesoftware.Steam", "-applaunch", STEAM_APP_ID];
    log_launch(&format!("Steam (flatpak): {}", flatpak_args.join(" ")));
    if spawn_detached("flatpak", &flatpak_args) {
        return Ok(true);
    }

    Ok(false)
}

fn ensure_steam_client_running() -> Result<bool> {
    if is_steam_client_running() {
        log_launch("Steam client already running");
        return Ok(true);
    }

    log_launch("Steam client not running; attempting to start it");
    let steam = match find_steam_executable() {
        Some(path) => path,
        None => return Ok(false),
    };

    if !spawn_detached(&steam.to_string_lossy(), &[]) {
        return Ok(false);
    }

    const WAIT_MS: u64 = 45_000;
    const POLL_MS: u64 = 500;
    let mut waited = 0_u64;
    while waited < WAIT_MS {
        if is_steam_client_running() {
            log_launch(&format!("Steam client ready after {waited}ms"));
            return Ok(true);
        }
        std::thread::sleep(std::time::Duration::from_millis(POLL_MS));
        waited += POLL_MS;
    }

    log_launch("Timed out waiting for Steam client to start");
    Ok(false)
}

fn is_steam_client_running() -> bool {
    if pgrep_matches("steam") {
        return true;
    }
    // Wrapper / Flatpak / distro-specific process names.
    pgrep_full_match("steam.sh") || pgrep_full_match("Steam")
}

fn pgrep_full_match(pattern: &str) -> bool {
    Command::new("pgrep")
        .args(["-f", pattern])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn pgrep_matches(pattern: &str) -> bool {
    Command::new("pgrep")
        .args(["-x", pattern])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn write_steam_appid(work_dir: &Path) -> Result<()> {
    fs::write(work_dir.join("steam_appid.txt"), format!("{STEAM_APP_ID}\n"))?;
    Ok(())
}

/// Injects Steam client / overlay hooks into the Proton process tree.
fn apply_steam_overlay_env(command: &mut Command, steam_root: &Path) {
    command
        .env("SteamAppId", STEAM_APP_ID)
        .env("SteamGameId", STEAM_APP_ID)
        .env("SteamLaunchAppId", STEAM_APP_ID);

    let overlay_64 = steam_root.join("ubuntu12_64/gameoverlayrenderer.so");
    if overlay_64.is_file() {
        command.env("LD_PRELOAD", overlay_64.to_string_lossy().into_owned());
    }
}

fn resolve_compat_data(paths: &GamePaths) -> Option<PathBuf> {
    if let Some(prefix) = &paths.proton_prefix {
        if let Some(compat) = prefix.parent() {
            if compat.ends_with(STEAM_APP_ID) && compat.join("pfx").is_dir() {
                return Some(compat.to_path_buf());
            }
        }
    }

    let candidate = paths
        .steam_library
        .join("steamapps/compatdata")
        .join(STEAM_APP_ID);
    if candidate.join("pfx").is_dir() {
        return Some(candidate);
    }

    find_compat_data_in_libraries().ok().flatten()
}

fn find_compat_data_in_libraries() -> Result<Option<PathBuf>> {
    let libraries = crate::paths::discover_steam_libraries()?;
    Ok(crate::paths::discover_proton_compat_data(&libraries))
}

fn find_steam_client_root() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        for candidate in [home.join(".steam/steam"), home.join(".local/share/Steam")] {
            if candidate.join("steamapps").is_dir() {
                return Some(candidate);
            }
        }
    }
    None
}

fn find_proton_tool(compat_data: &Path, steam_root: &Path) -> Option<PathBuf> {
    if let Some(path) = parse_compat_install_path(compat_data) {
        if path.join("proton").is_file() {
            return Some(path);
        }
    }

    if let Some(path) = parse_config_info_proton(compat_data) {
        if path.join("proton").is_file() {
            return Some(path);
        }
    }

    let common = steam_root.join("steamapps/common");
    let mut candidates: Vec<PathBuf> = fs::read_dir(&common)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("Proton"))
        })
        .collect();

    candidates.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    candidates.into_iter().find(|path| path.join("proton").is_file())
}

fn parse_compat_install_path(compat_data: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(compat_data.join("config.vdf")).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("\"install_path\"") {
            if let Some(path) = extract_quoted_value(rest) {
                return Some(PathBuf::from(path));
            }
        }
    }
    None
}

/// Newer Steam builds store Proton paths in `config_info` instead of `config.vdf`.
fn parse_config_info_proton(compat_data: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(compat_data.join("config_info")).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(path) = proton_dir_from_steam_path_line(trimmed) {
            return Some(path);
        }
    }
    None
}

fn proton_dir_from_steam_path_line(line: &str) -> Option<PathBuf> {
    const MARKER: &str = "/steamapps/common/";
    let marker_idx = line.find(MARKER)?;
    let after = &line[marker_idx + MARKER.len()..];
    let slash = after.find('/')?;
    let proton_name = &after[..slash];
    if !proton_name.starts_with("Proton") {
        return None;
    }
    let steam_root = &line[..marker_idx];
    Some(
        PathBuf::from(steam_root)
            .join("steamapps/common")
            .join(proton_name),
    )
}

fn extract_quoted_value(rest: &str) -> Option<String> {
    let mut parts = rest.split('"').filter(|part| !part.trim().is_empty());
    parts.next().map(ToString::to_string)
}

fn spawn_detached(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn find_steam_executable() -> Option<PathBuf> {
    if let Ok(path) = which_command("steam") {
        return Some(path);
    }

    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    [
        home.join(".local/share/Steam/ubuntu12_32/steam"),
        home.join(".local/share/Steam/steam.sh"),
        home.join(".steam/steam/ubuntu12_32/steam"),
        home.join(".steam/root/ubuntu12_32/steam"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

fn which_command(name: &str) -> std::io::Result<PathBuf> {
    let output = Command::new("which").arg(name).output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{name} not found"),
        ));
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

fn log_launch(message: &str) {
    let path = default_config_dir().join("launch.log");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_module_args_for_enabled_mods() {
        let args = build_module_launch_args(&[
            LoadOrderEntry {
                module_id: "Native".into(),
                enabled: true,
            },
            LoadOrderEntry {
                module_id: "Bannerlord.Harmony".into(),
                enabled: true,
            },
            LoadOrderEntry {
                module_id: "Sandbox".into(),
                enabled: false,
            },
        ]);

        assert_eq!(args[0], "/singleplayer");
        assert_eq!(
            args[1],
            "_MODULES_*Native*Bannerlord.Harmony*_MODULES_"
        );
    }

    #[test]
    fn parses_proton_dir_from_config_info_path() {
        let line = "/home/user/.steam/steam/steamapps/common/Proton 9.0 (Beta)/files/share/fonts/";
        let path = proton_dir_from_steam_path_line(line).unwrap();
        assert_eq!(
            path,
            PathBuf::from("/home/user/.steam/steam/steamapps/common/Proton 9.0 (Beta)")
        );
    }
}
