use cosmic_text::{
    Attrs, Buffer, Color as TextColor, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

use crate::{action::Action, config::Style, layout::ButtonLayout};

pub(crate) struct TextSpec<'a> {
    pub(crate) text: &'a str,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) size: f32,
    pub(crate) rgb: (u8, u8, u8),
}

pub(crate) fn draw_action_button(
    pixmap: &mut Pixmap,
    layout: ButtonLayout,
    action: Action,
    active: bool,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
) {
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

    let (icon, label, shortcut) = action.text();
    let center_x = layout.x + layout.width / 2.0;
    draw_text(
        font_system,
        swash_cache,
        pixmap,
        TextSpec {
            text: icon,
            x: center_x - 17.0,
            y: layout.y + layout.height * 0.08,
            size: 34.0,
            rgb: action.color(),
        },
    );
    draw_text(
        font_system,
        swash_cache,
        pixmap,
        TextSpec {
            text: label,
            x: center_x - label.len() as f32 * 3.5,
            y: layout.y + layout.height * 0.62,
            size: 13.0,
            rgb: (169, 177, 214),
        },
    );
    draw_text(
        font_system,
        swash_cache,
        pixmap,
        TextSpec {
            text: shortcut,
            x: center_x - shortcut.len() as f32 * 2.7,
            y: layout.y + layout.height * 0.82,
            size: 11.0,
            rgb: (86, 95, 135),
        },
    );
}

pub(crate) fn draw_confirmation_hint(
    pixmap: &mut Pixmap,
    style: &Style,
    surface_width: f32,
    surface_height: f32,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(247, 118, 142, 220));
    pixmap.fill_rect(
        Rect::from_xywh(
            style.padding * 2.0,
            style.padding * 2.0,
            surface_width - style.padding * 4.0,
            style.padding / 2.0,
        )
        .unwrap(),
        &paint,
        Transform::identity(),
        None,
    );
    draw_text(
        font_system,
        swash_cache,
        pixmap,
        TextSpec {
            text: "Press again to confirm",
            x: surface_width / 2.0 - 62.0,
            y: surface_height - style.padding * 1.35,
            size: 11.0,
            rgb: (255, 255, 255),
        },
    );
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
