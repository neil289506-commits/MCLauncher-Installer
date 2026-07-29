// Hide the console window in release builds (curl / IDM child processes are hidden
// separately via CREATE_NO_WINDOW, see downloader.rs).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod downloader;
mod i18n;
mod idm;
mod terminal;
mod theme;

use eframe::egui;

/// Icon bytes, embedded into the binary at compile time so the window/taskbar
/// icon keeps working regardless of where the built `.exe` is run from
/// (the exe's own file icon is embedded separately at compile time via
/// `build.rs` + `winres`, see that file).
static ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

fn load_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(ICON_BYTES).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([880.0, 620.0])
        .with_min_inner_size([720.0, 480.0])
        .with_title("Minecraft Launcher Installer");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Minecraft Launcher Installer",
        options,
        Box::new(|cc| {
            theme::setup_fonts(&cc.egui_ctx);
            theme::setup_style(&cc.egui_ctx);
            Ok(Box::new(app::App::new()))
        }),
    )
}
