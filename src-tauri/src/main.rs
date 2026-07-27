#[cfg(target_os = "linux")]
mod linux_graphics;

fn main() {
    #[cfg(target_os = "linux")]
    linux_graphics::apply();

    mb2_mod_launcher_lib::run();
}
