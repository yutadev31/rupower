use std::{error::Error, process::Command};

use cosmic_text::{
    Attrs, Buffer, Color as TextColor, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use shell_surface::{
    Anchors, Backend, InputEvent, KeyboardInteractivity, Layer, MouseButton, Shell, Size,
    SurfaceConfig, SurfaceId,
};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

const PADDING: f32 = 16.0;
const ICON_AREA: (f32, f32) = (104.0, 124.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Poweroff,
    Reboot,
    Suspend,
    Logout,
    Lock,
}

impl Action {
    fn all() -> [Self; 5] {
        [
            Self::Poweroff,
            Self::Reboot,
            Self::Suspend,
            Self::Logout,
            Self::Lock,
        ]
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

struct PowerMenu {
    surface: SurfaceConfig,
    font_system: FontSystem,
    swash_cache: SwashCache,
    hovered: Option<Action>,
    confirm: Option<Action>,
    shift: bool,
}

impl PowerMenu {
    fn new() -> Self {
        let mut surface = SurfaceConfig::new("rupower", menu_size());
        surface.layer = Layer::Overlay;
        surface.keyboard_interactivity = KeyboardInteractivity::Exclusive;
        surface.anchors = Anchors::empty();
        // Keep this as a normal X11 window so i3 can place it according to its
        // usual floating/window rules. Wayland ignores this field.
        surface.override_redirect = false;
        Self {
            surface,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            hovered: None,
            confirm: None,
            shift: false,
        }
    }

    fn action_at(&self, x: f64, y: f64) -> Option<Action> {
        for (index, action) in Action::all().into_iter().enumerate() {
            let (button_x, button_y, button_width, button_height) = button_layout(index);
            if (button_x as f64..=(button_x + button_width) as f64).contains(&x)
                && (button_y as f64..=(button_y + button_height) as f64).contains(&y)
            {
                return Some(action);
            }
        }
        None
    }

    fn run_action(&mut self, action: Action) {
        let command = match action {
            Action::Poweroff => ("systemctl", vec!["poweroff"]),
            Action::Reboot => ("systemctl", vec!["reboot"]),
            Action::Suspend => ("systemctl", vec!["suspend"]),
            Action::Lock => ("loginctl", vec!["lock-session"]),
            Action::Logout => {
                if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
                    // Hyprland's Lua config treats `dispatch` arguments as a
                    // Lua expression. The legacy `dispatch exit` form makes
                    // `exit` a bare identifier and is rejected as nil.
                    ("hyprctl", vec!["dispatch", "hl.dsp.exit()"])
                } else if std::env::var_os("SWAYSOCK").is_some() {
                    ("swaymsg", vec!["exit"])
                } else {
                    ("i3-msg", vec!["exit"])
                }
            }
        };
        if let Err(error) = Command::new(command.0).args(command.1).spawn() {
            eprintln!("rupower: failed to start {}: {error}", command.0);
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
                PADDING,
                PADDING,
                surface_width - PADDING * 2.0,
                surface_height - PADDING * 2.0,
            )
            .unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
        for (index, action) in Action::all().into_iter().enumerate() {
            let (button_x, button_y, button_width, button_height) = button_layout(index);
            draw_button(
                &mut pixmap,
                button_x,
                button_y,
                button_width,
                button_height,
                action,
                self.hovered == Some(action) || self.confirm == Some(action),
            );
            let (icon, label, shortcut) = action.text();
            let center_x = button_x + button_width / 2.0;
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                icon,
                center_x - 17.0,
                button_y + button_height * 0.08,
                34.0,
                action.color(),
            );
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                label,
                center_x - label.len() as f32 * 3.5,
                button_y + button_height * 0.62,
                13.0,
                (169, 177, 214),
            );
            draw_text(
                &mut self.font_system,
                &mut self.swash_cache,
                &mut pixmap,
                shortcut,
                center_x - shortcut.len() as f32 * 2.7,
                button_y + button_height * 0.82,
                11.0,
                (86, 95, 135),
            );
        }
        if self.confirm.is_some() {
            paint.set_color(Color::from_rgba8(247, 118, 142, 220));
            pixmap.fill_rect(
                Rect::from_xywh(
                    PADDING * 2.0,
                    PADDING * 2.0,
                    surface_width - PADDING * 4.0,
                    PADDING / 2.0,
                )
                .unwrap(),
                &paint,
                Transform::identity(),
                None,
            );
        }
        // shell-surface expects native little-endian ARGB8888 (BGRA bytes).
        let mut pixels = pixmap.data().to_vec();
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        pixels.resize(size.width as usize * size.height as usize * 4, 0);
        Ok(pixels)
    }

    fn handle_event(&mut self, _surface: SurfaceId, event: InputEvent) {
        match event {
            InputEvent::CloseRequested => {}
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
                        let action = match keycode {
                            25 => Some(Action::Poweroff), // P
                            19 => Some(Action::Reboot),   // R
                            31 => Some(Action::Suspend),  // S
                            38 => Some(Action::Logout),   // L
                            37 => Some(Action::Lock),     // K
                            _ => None,
                        };
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

fn draw_text(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    pixmap: &mut Pixmap,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    rgb: (u8, u8, u8),
) {
    let mut buffer = Buffer::new(font_system, Metrics::new(size, size * 1.2));
    let mut buffer = buffer.borrow_with(font_system);
    buffer.set_size(Some(180.0), Some(size * 1.5));
    let attrs = Attrs::new().family(Family::Name("Hack Nerd Font"));
    buffer.set_text(text, &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(true);
    let width = pixmap.width();
    let height = pixmap.height();
    buffer.draw(
        swash_cache,
        TextColor::rgb(rgb.0, rgb.1, rgb.2),
        |px, py, w, h, color| {
            let data = pixmap.data_mut();
            for dy in 0..h {
                for dx in 0..w {
                    let xx = x as i32 + px + dx as i32;
                    let yy = y as i32 + py + dy as i32;
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

fn menu_size() -> Size {
    let action_count = Action::all().len() as f32;
    let menu_width = action_count * ICON_AREA.0 + (action_count + 3.0) * PADDING;
    let menu_height = ICON_AREA.1 + PADDING * 4.0;
    Size::new(menu_width as u32, menu_height as u32)
}

fn button_layout(index: usize) -> (f32, f32, f32, f32) {
    let button_width = ICON_AREA.0;
    let button_height = ICON_AREA.1;
    let x = PADDING * 2.0 + index as f32 * (button_width + PADDING);
    let y = PADDING * 2.0;
    (x, y, button_width, button_height)
}

fn draw_button(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    _action: Action,
    active: bool,
) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(
        if active { 59 } else { 41 },
        if active { 66 } else { 46 },
        if active { 97 } else { 66 },
        255,
    ));
    pixmap.fill_rect(
        Rect::from_xywh(x, y, width, height).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = PowerMenu::new();
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        shell_surface::backend::wayland::WaylandBackend.run(&mut app)?;
    } else if std::env::var_os("DISPLAY").is_some() {
        shell_surface::backend::x11::X11Backend.run(&mut app)?;
    } else {
        return Err("neither WAYLAND_DISPLAY nor DISPLAY is set".into());
    }
    Ok(())
}
