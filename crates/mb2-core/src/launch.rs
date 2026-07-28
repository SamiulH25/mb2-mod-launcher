use crate::error::{Mb2Error, Result};
use crate::paths::STEAM_APP_ID;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn steam_launch_url() -> String {
    format!("steam://run/{STEAM_APP_ID}")
}

/// Saves load order if needed, then asks Steam to start Bannerlord (Proton/Wine).
pub fn launch_via_steam() -> Result<()> {
    let url = steam_launch_url();

    if spawn_detached("xdg-open", &[&url]) {
        return Ok(());
    }

    if let Some(steam) = find_steam_executable() {
        let steam_str = steam.to_string_lossy();
        if spawn_detached(&steam_str, &["-applaunch", STEAM_APP_ID]) {
            return Ok(());
        }
        if spawn_detached(&steam_str, &[&url]) {
            return Ok(());
        }
    }

    if spawn_detached(
        "flatpak",
        &[
            "run",
            "com.valvesoftware.Steam",
            "-applaunch",
            STEAM_APP_ID,
        ],
    ) {
        return Ok(());
    }

    Err(Mb2Error::GameNotFound(
        "Could not launch Bannerlord through Steam. Make sure Steam is installed and try starting Steam first."
            .into(),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_url_uses_bannerlord_app_id() {
        assert_eq!(steam_launch_url(), "steam://run/261550");
    }
}
