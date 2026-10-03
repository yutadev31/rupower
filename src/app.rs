use std::{error::Error, process::Command};

use cosmic_text::{FontSystem, SwashCache};
use shell_surface::{
    Anchors, InputEvent, KeyboardInteractivity, Layer, MouseButton, Shell, Size, SurfaceConfig,
    SurfaceId,
};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

use crate::{
    action::Action,
    config::Config,
    layout::{button_layout, menu_size},
    render::{draw_action_button, draw_confirmation_hint},
};

pub(crate) struct PowerMenu {
    config: Config,
    surface: SurfaceConfig,
    font_system: FontSystem,
    swash_cache: SwashCache,
    hovered: Option<Action>,
    confirm: Option<Action>,
    shift: bool,
}

impl PowerMenu {
    pub(crate) fn new(config: Config) -> Self {
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
        Action::enabled(&self.config)
            .enumerate()
            .find(|(index, _)| button_layout(*index, &self.config.style).contains(x, y))
            .map(|(_, action)| action)
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
        self.draw_background(&mut pixmap, surface_width, surface_height);

        for (index, action) in Action::enabled(&self.config).enumerate() {
            draw_action_button(
                &mut pixmap,
                button_layout(index, &self.config.style),
                action,
                self.hovered == Some(action) || self.confirm == Some(action),
                &mut self.font_system,
                &mut self.swash_cache,
            );
        }

        if self.confirm.is_some() {
            draw_confirmation_hint(
                &mut pixmap,
                &self.config.style,
                surface_width,
                surface_height,
                &mut self.font_system,
                &mut self.swash_cache,
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
            InputEvent::Key { keycode, pressed } => match keycode {
                42 | 54 => self.shift = pressed,
                _ if pressed && self.shift => {
                    if let Some(action) = Action::from_keycode(keycode) {
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
            },
            _ => {}
        }
    }
}

impl PowerMenu {
    fn draw_background(&self, pixmap: &mut Pixmap, width: f32, height: f32) {
        // Keep the area outside the menu transparent. The pixmap starts as
        // transparent black, so there is no full-surface dimming frame.
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(36, 40, 59, 255));
        pixmap.fill_rect(
            Rect::from_xywh(
                self.config.style.padding,
                self.config.style.padding,
                width - self.config.style.padding * 2.0,
                height - self.config.style.padding * 2.0,
            )
            .unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
    }
}
