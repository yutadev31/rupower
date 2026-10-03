use shell_surface::Size;

use crate::{
    action::Action,
    config::{Config, Style},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ButtonLayout {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl ButtonLayout {
    pub(crate) fn contains(self, x: f64, y: f64) -> bool {
        (self.x as f64..=(self.x + self.width) as f64).contains(&x)
            && (self.y as f64..=(self.y + self.height) as f64).contains(&y)
    }
}

pub(crate) fn menu_size(config: &Config) -> Size {
    let action_count = Action::enabled(config).count() as f32;
    let style = &config.style;
    let menu_width = action_count * style.button_width + (action_count + 3.0) * style.padding;
    let menu_height = style.button_height + style.padding * 4.0;
    Size::new(menu_width as u32, menu_height as u32)
}

pub(crate) fn button_layout(index: usize, config: &Style) -> ButtonLayout {
    ButtonLayout {
        x: config.padding * 2.0 + index as f32 * (config.button_width + config.padding),
        y: config.padding * 2.0,
        width: config.button_width,
        height: config.button_height,
    }
}
