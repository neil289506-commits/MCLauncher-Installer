//! Black background + white text theme, plus font setup.
//!
//! The font is embedded into the executable at **compile time** via
//! `include_bytes!`, so it keeps working no matter where the released
//! `.exe` is launched from. (This replaces the earlier runtime
//! `std::fs::read("assets/msjh.ttf")` approach, which broke after
//! `cargo build --release` because `target\release\assets\` didn't
//! contain a copy of the font file.)
//!
//! **Before building**, place your font at `assets/msjh.ttf`
//! (Microsoft JhengHei, or any other `.ttf` you prefer — just rename it).
//! Since the font is now compiled in, the file must exist at *build* time,
//! not just at runtime — if it's missing, `cargo build` will fail with a
//! clear "file not found" error pointing at `assets/msjh.ttf`.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

/// Font bytes, embedded into the binary at compile time.
static FONT_BYTES: &[u8] = include_bytes!("../assets/msjh.ttf");

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts
        .font_data
        .insert("msjh".to_owned(), FontData::from_static(FONT_BYTES).into());

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "msjh".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("msjh".to_owned());

    ctx.set_fonts(fonts);
}

pub fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    // Minecraft emerald-green accent (used for highlights/buttons only; everything
    // else follows the black-background / white-text theme).
    let accent = egui::Color32::from_rgb(72, 191, 116);
    let white = egui::Color32::from_rgb(240, 240, 240);

    visuals.selection.bg_fill = accent;
    visuals.hyperlink_color = accent;

    // Black background
    visuals.window_fill = egui::Color32::BLACK;
    visuals.panel_fill = egui::Color32::BLACK;
    visuals.faint_bg_color = egui::Color32::from_rgb(10, 10, 10);
    visuals.extreme_bg_color = egui::Color32::BLACK;

    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(14, 14, 14);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(20, 20, 20);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(34, 34, 34);
    visuals.widgets.active.bg_fill = accent;

    // White text. Note RichText::strong() also pulls its color from the
    // "active" style, so it's set to white here too — otherwise the dark
    // text meant to contrast against a pressed (green) button would also
    // make ordinary emphasized text invisible on the black background.
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, white);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, white);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    // Thin borders so cards still have some depth against the pure black background.
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 40, 40));
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 45, 45));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, accent);

    visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    visuals.window_rounding = egui::Rounding::same(12.0);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16.0);

    ctx.set_style(style);
}
