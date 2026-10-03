use std::{error::Error, process::Command};

use cosmic_text::{
    Attrs, Buffer, Color as TextColor, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use shell_surface::{
    Anchors, Backend, InputEvent, KeyboardInteractivity, Layer, MouseButton, Shell, Size,
    SurfaceConfig, SurfaceId,
};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

mod config;

use config::{Config, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Poweroff,
    Reboot,
    Suspend,
    Logout,
    Lock,
}

impl Action {
    const ALL: [Self; 5] = [
        Self::Poweroff,
        Self::Reboot,
        Self::Suspend,
        Self::Logout,
        Self::Lock,
    ];

    fn all() -> impl Iterator<Item = Self> {
        Self::ALL.into_iter()
    }

    fn command(self) -> CommandSpec {
        match self {
            Self::Poweroff => CommandSpec::new("poweroff", &[]),
            Self::Reboot => CommandSpec::new("reboot", &[]),
            Self::Suspend => CommandSpec::new("systemctl", &["suspend"]),
            Self::Lock => CommandSpec::new("loginctl", &["lock-session"]),
            Self::Logout if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() => {
                CommandSpec::new("hyprctl", &["dispatch", "hl.dsp.exit()"])
            }
            Self::Logout if std::env::var_os("SWAYSOCK").is_some() => {
                CommandSpec::new("swaymsg", &["exit"])
            }
            Self::Logout => CommandSpec::new("i3-msg", &["exit"]),
        }
    }

    fn keycode(self) -> u32 {
        match self {
            Self::Poweroff => 25,
            Self::Reboot => 19,
            Self::Suspend => 31,
            Self::Logout => 38,
            Self::Lock => 37,
        }
    }

    fn from_keycode(keycode: u32) -> Option<Self> {
        Self::all().find(|action| action.keycode() == keycode)
    }

    fn is_enabled(self, config: &Config) -> bool {
        match self {
            Self::Poweroff => config.actions.poweroff,
            Self::Reboot => config.actions.reboot,
            Self::Suspend => config.actions.suspend,
            Self::Logout => config.actions.logout,
            Self::Lock => config.actions.lock,
        }
    }

    fn enabled(config: &Config) -> impl Iterator<Item = Self> {
        Self::all().filter(move |action| action.is_enabled(config))
    }

    fn color(self) -> (u8, u8, u8) {
        match self {
            Self::Poweroff => (247, 118, 142),
            Self::Reboot => (224, 175, 104),
            Self::Suspend => (125, 207, 255),
            Self::Logout => (187, 154, 247),
            Self::Lock => (158, 206, 106),
        }
    }

    fn text(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Poweroff => ("\u{f011}", "Power off", "SHIFT+P"),
            Self::Reboot => ("\u{f021}", "Reboot", "SHIFT+R"),
            Self::Suspend => ("\u{f186}", "Suspend", "SHIFT+S"),
            Self::Logout => ("\u{f2f5}", "Logout", "SHIFT+L"),
            Self::Lock => ("\u{f023}", "Lock", "SHIFT+K"),
        }
    }
}

struct CommandSpec {
    program: &'static str,
    args: &'static [&'static str],
}

impl CommandSpec {
    const fn new(program: &'static str, args: &'static [&'static str]) -> Self {
        Self { program, args }
    }
}

struct PowerMenu {
    config: Config,
    surface: SurfaceConfig,
    font_system: FontSystem,
    swash_cache: SwashCache,
    hovered: Option<Action>,
    confirm: Option<Action>,
    shift: bool,
}

impl PowerMenu {
    fn new(config: Config) -> Self {
        let mut surface = SurfaceConfig::new("rupower", menu_size(&config));
        surface.layer = Layer::Overlay;
        surface.keyboard_interactivity = KeyboardInteractivity::Exclusive;
        surface.anchors = Anchors::empty();
        // Keep this as a normal X11 window so i3 can place it according to its
        // usual floating/window rules. Wayland ignores this field.
        surface.override_redirect = false;
        Self {
            config,
            surface,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            hovered: None,
            confirm: None,
            shift: false,
        }
    }

    fn action_at(&self, x: f64, y: f64) -> Option<Action> {
        for (index, action) in Action::enabled(&self.config).enumerate() {
            if button_layout(index, &self.config.style).contains(x, y) {
                return Some(action);
            }
        }
        None
    }

    fn run_action(&mut self, action: Action) {
        let command = action.command();
        if let Err(error) = Command::new(command.program).args(command.args).spawn() {
            eprintln!("rupower: failed to start {}: {error}", command.program);
            return;
        }
        std::process::exit(0);
    }
}

impl Shell for PowerMenu {
    fn surface_configs(&self) -> &[SurfaceConfig] {
        std::slice::from_ref(&self.surface)
    }

    fn render(
        &mut self,
        _surface: SurfaceId,
        size: Size,
        _output: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut pixmap = Pixmap::new(size.width, size.height).ok_or("invalid surface size")?;
        let surface_width = size.width as f32;
        let surface_height = size.height as f32;
        // Keep the area outside the menu transparent. The pixmap starts as
        // transparent black, so there is no full-surface dimming frame.
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(36, 40, 59, 255));
        pixmap.fill_rect(
            Rect::from_xywh(
                self.config.style.padding,
                self.config.style.padding,
                surface_width - self.config.style.padding * 2.0,
                surface_height - self.config.style.padding * 2.0,
            )
            .unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
        for (index, action) in Action::enabled(&self.config).enumerate() {
            let layout = button_layout(index, &self.config.style);
            draw_button(
                &mut pixmap,
                layout,
                self.hovered == Some(action) || self.confirm == Some(action),
            );
            let (icon, label, shortcut) = action.text();
            let center_x = layout.x + layout.width / 2.0;
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                TextSpec {
                    text: icon,
                    x: center_x - 17.0,
                    y: layout.y + layout.height * 0.08,
                    size: 34.0,
                    rgb: action.color(),
                },
            );
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                TextSpec {
                    text: label,
                    x: center_x - label.len() as f32 * 3.5,
                    y: layout.y + layout.height * 0.62,
                    size: 13.0,
                    rgb: (169, 177, 214),
                },
            );
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                TextSpec {
                    text: shortcut,
                    x: center_x - shortcut.len() as f32 * 2.7,
                    y: layout.y + layout.height * 0.82,
                    size: 11.0,
                    rgb: (86, 95, 135),
                },
            );
        }
        if self.confirm.is_some() {
            paint.set_color(Color::from_rgba8(247, 118, 142, 220));
            pixmap.fill_rect(
                Rect::from_xywh(
                    self.config.style.padding * 2.0,
                    self.config.style.padding * 2.0,
                    surface_width - self.config.style.padding * 4.0,
                    self.config.style.padding / 2.0,
                )
                .unwrap(),
                &paint,
                Transform::identity(),
                None,
            );
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                TextSpec {
                    text: "Press again to confirm",
                    x: surface_width / 2.0 - 62.0,
                    y: surface_height - self.config.style.padding * 1.35,
                    size: 11.0,
                    rgb: (255, 255, 255),
                },
            );
        }
        // shell-surface expects native little-endian ARGB8888 (BGRA bytes).
        let mut pixels = pixmap.data().to_vec();
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
        pixels.resize(size.width as usize * size.height as usize * 4, 0);
        Ok(pixels)
    }

    fn handle_event(&mut self, _surface: SurfaceId, event: InputEvent) {
        match event {
            InputEvent::CloseRequested => std::process::exit(0),
            InputEvent::PointerMotion { position, .. }
            | InputEvent::PointerEnter { position, .. } => {
                self.hovered = self.action_at(position.x, position.y)
            }
            InputEvent::PointerLeave => self.hovered = None,
            InputEvent::PointerButton {
                position,
                button: MouseButton::Left,
                pressed: true,
                ..
            } => {
                if let Some(action) = self.action_at(position.x, position.y) {
                    if self.confirm == Some(action) {
                        self.run_action(action);
                    } else {
                        self.confirm = Some(action);
                    }
                } else {
                    self.confirm = None;
                }
            }
            InputEvent::Key { keycode, pressed } => {
                match keycode {
                    42 | 54 => self.shift = pressed,
                    _ if pressed && self.shift => {
                        let action = Action::from_keycode(keycode);
                        if let Some(action) = action {
                            if self.confirm == Some(action) {
                                self.run_action(action);
                            } else {
                                self.confirm = Some(action);
                            }
                        }
                    }
                    1 | 9 | 24 if pressed => std::process::exit(0), // Escape / q / x
                    28 if pressed => {
                        if let Some(action) = self.confirm {
                            self.run_action(action);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

struct TextSpec<'a> {
    text: &'a str,
    x: f32,
    y: f32,
    size: f32,
    rgb: (u8, u8, u8),
}

#[derive(Clone, Copy, Debug)]
struct ButtonLayout {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ButtonLayout {
    fn contains(self, x: f64, y: f64) -> bool {
        (self.x as f64..=(self.x + self.width) as f64).contains(&x)
            && (self.y as f64..=(self.y + self.height) as f64).contains(&y)
    }
}

fn draw_text(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    pixmap: &mut Pixmap,
    spec: TextSpec<'_>,
) {
    let mut buffer = Buffer::new(font_system, Metrics::new(spec.size, spec.size * 1.2));
    let mut buffer = buffer.borrow_with(font_system);
    buffer.set_size(Some(180.0), Some(spec.size * 1.5));
    let attrs = Attrs::new().family(Family::Name("Hack Nerd Font"));
    buffer.set_text(spec.text, &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(true);
    let width = pixmap.width();
    let height = pixmap.height();
    buffer.draw(
        swash_cache,
        TextColor::rgb(spec.rgb.0, spec.rgb.1, spec.rgb.2),
        |px, py, w, h, color| {
            let data = pixmap.data_mut();
            for dy in 0..h {
                for dx in 0..w {
                    let xx = spec.x as i32 + px + dx as i32;
                    let yy = spec.y as i32 + py + dy as i32;
                    if xx < 0 || yy < 0 || xx >= width as i32 || yy >= height as i32 {
                        continue;
                    }
                    let offset = ((yy as u32 * width + xx as u32) * 4) as usize;
                    let alpha = color.a() as u32;
                    let inv = 255 - alpha;
                    data[offset] =
                        ((color.r() as u32 * alpha + data[offset] as u32 * inv) / 255) as u8;
                    data[offset + 1] =
                        ((color.g() as u32 * alpha + data[offset + 1] as u32 * inv) / 255) as u8;
                    data[offset + 2] =
                        ((color.b() as u32 * alpha + data[offset + 2] as u32 * inv) / 255) as u8;
                    data[offset + 3] = (alpha + data[offset + 3] as u32 * inv / 255).min(255) as u8;
                }
            }
        },
    );
}

fn menu_size(config: &Config) -> Size {
    let action_count = Action::enabled(config).count() as f32;
    let style = &config.style;
    let menu_width = action_count * style.button_width + (action_count + 3.0) * style.padding;
    let menu_height = style.button_height + style.padding * 4.0;
    Size::new(menu_width as u32, menu_height as u32)
}

fn button_layout(index: usize, config: &Style) -> ButtonLayout {
    let button_width = config.button_width;
    let button_height = config.button_height;
    let x = config.padding * 2.0 + index as f32 * (button_width + config.padding);
    let y = config.padding * 2.0;
    ButtonLayout {
        x,
        y,
        width: button_width,
        height: button_height,
    }
}

fn draw_button(pixmap: &mut Pixmap, layout: ButtonLayout, active: bool) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(
        if active { 59 } else { 41 },
        if active { 66 } else { 46 },
        if active { 97 } else { 66 },
        255,
    ));
    pixmap.fill_rect(
        Rect::from_xywh(layout.x, layout.y, layout.width, layout.height).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );
}

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
