use std::error::Error;

use shell_surface::Backend;

mod action;
mod app;
mod config;
mod layout;
mod render;

use app::PowerMenu;
use config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = PowerMenu::new(Config::load()?);
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        shell_surface::backend::wayland::WaylandBackend.run(&mut app)?;
    } else if std::env::var_os("DISPLAY").is_some() {
        shell_surface::backend::x11::X11Backend.run(&mut app)?;
    } else {
        return Err("neither WAYLAND_DISPLAY nor DISPLAY is set".into());
    }
    Ok(())
}
