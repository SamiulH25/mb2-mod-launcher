//! WebKitGTK + Wayland workarounds for Linux (see tauri-apps/tauri#9394).

/// Apply graphics driver workarounds before GTK/WebKit initializes.
///
/// Skipped when `MB2_DISABLE_GRAPHICS_FIX=1` or when the user already set a
/// known override in the environment.
pub fn apply() {
    if std::env::var("MB2_DISABLE_GRAPHICS_FIX").as_deref() == Ok("1") {
        return;
    }

    set_default_env("__NV_DISABLE_EXPLICIT_SYNC", "1");
    set_default_env("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
}

fn set_default_env(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        // SAFETY: called at process start before other threads exist.
        unsafe { std::env::set_var(key, value) };
    }
}
