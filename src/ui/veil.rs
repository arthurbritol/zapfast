//! The privacy blur: names, messages and pictures stay covered until the
//! pointer, or keyboard focus, is on them.
//!
//! egui has no image filter, so this is an opaque cover, not a real blur. A
//! covered row never lays out its real text: the text is not drawn at all,
//! so nothing is left behind the cover to show through. Callers draw bars
//! and discs in place of names, previews and pictures, and only when
//! [`covered`] says so; with the setting off nothing here costs anything.

use egui::{Color32, CornerRadius, Pos2, Rect, Response, Ui, Vec2};

use crate::theme::Palette;

/// Whether the pointer rests on `rect` in this layer: a menu or dialog open
/// above it does not count.
pub fn pointer_over(ui: &Ui, rect: Rect) -> bool {
    let Some(pos) = ui.input(|input| input.pointer.hover_pos()) else {
        return false;
    };
    rect.contains(pos)
        && ui
            .ctx()
            .layer_id_at(pos)
            .is_none_or(|layer| layer == ui.layer_id())
}

/// Whether keyboard focus is on a widget inside `rect`.
pub fn focus_within(ui: &Ui, rect: Rect) -> bool {
    ui.memory(|memory| memory.focused())
        .and_then(|id| ui.ctx().read_response(id))
        .is_some_and(|focused| rect.contains(focused.rect.center()))
}

/// Whether what `response` stands for stays covered: the blur is on and
/// neither the pointer nor keyboard focus is on it.
pub fn covered(blur: bool, response: &Response) -> bool {
    blur && !(response.hovered() || response.has_focus() || response.is_pointer_button_down_on())
}

/// The cover's colour: opaque, between the surface and the dim text, so it
/// reads as a soft blot on both palettes without hiding the layout.
pub fn cover_color(palette: &Palette) -> Color32 {
    palette.surface.lerp_to_gamma(palette.dim, 0.45)
}

/// A bar standing in for one line of text, `width` by `height` from `pos`.
pub fn bar(ui: &Ui, palette: &Palette, pos: Pos2, width: f32, height: f32) {
    let rect = Rect::from_min_size(pos, Vec2::new(width.max(0.0), height));
    ui.painter().rect_filled(
        rect,
        CornerRadius::same((height / 2.0) as u8),
        cover_color(palette),
    );
}

/// A disc standing in for an avatar, filling `rect`.
pub fn disc(ui: &Ui, palette: &Palette, rect: Rect) {
    ui.painter().circle_filled(
        rect.center(),
        rect.width().min(rect.height()) / 2.0,
        cover_color(palette),
    );
}

/// An opaque cover over `rect`, for content already drawn or too varied to
/// stand in for, such as a picture or a message bubble.
pub fn cover(ui: &Ui, palette: &Palette, rect: Rect, radius: f32) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(radius.clamp(0.0, 255.0) as u8),
        cover_color(palette),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cover_is_opaque_in_both_palettes() {
        for palette in [Palette::dark(), Palette::light()] {
            let color = cover_color(&palette);
            assert_eq!(color.a(), 255);
            assert_ne!(color, palette.surface);
            assert_ne!(color, palette.text);
        }
    }

    #[test]
    fn nothing_is_covered_with_the_setting_off() {
        let ctx = egui::Context::default();
        let mut hovered = None;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let response = ui.allocate_response(Vec2::splat(20.0), egui::Sense::click());
            hovered = Some((covered(false, &response), covered(true, &response)));
        });
        output.textures_delta.clear();
        assert_eq!(hovered, Some((false, true)));
    }
}
