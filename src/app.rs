use eframe::egui::{self, Color32, RichText, Rounding, Vec2};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use crate::data::{self, LAUNCHERS};
use crate::downloader::{self, DownloadProgress};
use crate::i18n::{Lang, Strings};
use crate::idm;

/// Screen transition animation duration (seconds).
const TRANSITION_SECS: f32 = 0.22;

#[derive(Clone, Copy, PartialEq)]
enum VariantKind {
    Installer,
    Portable,
}

impl VariantKind {
    fn label(self, t: &Strings) -> &'static str {
        match self {
            VariantKind::Installer => t.installer_label,
            VariantKind::Portable => t.portable_label,
        }
    }
}

/// Which tool to hand the download off to. Only meaningful (and only shown
/// as a choice in the UI) when IDM is actually installed — otherwise curl
/// is always used, same as before.
#[derive(Clone, Copy, PartialEq)]
enum DownloadMethod {
    Idm,
    Curl,
}

enum Screen {
    List,
    Detail(usize),
    Downloading {
        launcher: String,
        variant: VariantKind,
        progress: Arc<DownloadProgress>,
        save_path: PathBuf,
    },
}

pub struct App {
    screen: Screen,
    idm_path: Option<PathBuf>,
    download_method: DownloadMethod,
    lang: Lang,
    toast: Option<(String, f32)>,
    save_dir: PathBuf,
    transition_start: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::List,
            idm_path: idm::detect_idm(),
            download_method: DownloadMethod::Idm,
            lang: Lang::En,
            toast: None,
            save_dir: default_download_dir(),
            transition_start: Instant::now(),
        }
    }

    /// The single entry point for switching screens; also resets the transition timer.
    fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.transition_start = Instant::now();
    }

    fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), 3.0));
    }

    fn start_download(
        &mut self,
        launcher_name: &str,
        kind: VariantKind,
        url: &str,
        filename_override: Option<&str>,
    ) {
        let t = self.lang.strings();
        let filename = filename_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| data::guess_filename(url));
        let use_idm = self.download_method == DownloadMethod::Idm && self.idm_path.is_some();

        if use_idm {
            let idm_exe = self.idm_path.clone().expect("checked is_some() above");
            let save_dir = self.save_dir.to_string_lossy().to_string();
            match idm::download_with_idm(&idm_exe, url, &save_dir, &filename) {
                Ok(()) => {
                    if kind == VariantKind::Installer {
                        self.show_toast(format!(
                            "{}{filename}{}",
                            t.handed_to_idm, t.handed_to_idm_manual_suffix
                        ));
                    } else {
                        self.show_toast(format!("{}{filename}", t.handed_to_idm));
                    }
                }
                Err(e) => self.show_toast(format!("{}{e}", t.idm_launch_failed)),
            }
        } else {
            let progress = DownloadProgress::new();
            let save_path = self.save_dir.join(&filename);
            downloader::start_download(url.to_string(), save_path.clone(), progress.clone());
            self.set_screen(Screen::Downloading {
                launcher: launcher_name.to_string(),
                variant: kind,
                progress,
                save_path,
            });
        }
    }
}

/// Ease-out-cubic easing function, for a smoother animation finish.
fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn default_download_dir() -> PathBuf {
    if let Some(user) = std::env::var_os("USERPROFILE") {
        let p = PathBuf::from(user).join("Downloads");
        if p.exists() {
            return p;
        }
    }
    std::env::current_dir().unwrap_or_default()
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = self.lang.strings();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if matches!(self.screen, Screen::Detail(_) | Screen::Downloading { .. })
                    && ui.button(t.back).clicked()
                {
                    self.set_screen(Screen::List);
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Minecraft Launcher Installer")
                        .heading()
                        .color(Color32::from_rgb(120, 220, 150)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Language switch — always available, top-right corner.
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.lang, Lang::En, "EN");
                        ui.selectable_value(&mut self.lang, Lang::ZhHans, "简");
                    });

                    ui.add_space(16.0);

                    if self.idm_path.is_some() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(t.idm_prefix).color(Color32::from_rgb(120, 220, 150)));
                            ui.selectable_value(&mut self.download_method, DownloadMethod::Idm, "IDM");
                            ui.selectable_value(&mut self.download_method, DownloadMethod::Curl, "curl");
                        });
                    } else {
                        ui.label(RichText::new(t.idm_not_found).color(Color32::from_rgb(200, 170, 90)));
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Transition progress: 0 = screen just switched, 1 = animation finished.
        let progress_t = self.transition_start.elapsed().as_secs_f32() / TRANSITION_SECS;
        let eased = ease_out_cubic(progress_t);
        let alpha = eased;
        let offset_y = (1.0 - eased) * 14.0;
        if progress_t < 1.0 {
            ctx.request_repaint();
        }

        match &self.screen {
            Screen::List => self.ui_list(ctx, alpha, offset_y),
            Screen::Detail(idx) => self.ui_detail(ctx, *idx, alpha, offset_y),
            Screen::Downloading { .. } => self.ui_downloading(ctx, alpha, offset_y),
        }

        if let Some((msg, remaining)) = &mut self.toast {
            egui::Area::new(egui::Id::new("toast"))
                .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, -20.0))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(40, 40, 44))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| ui.label(msg.as_str()));
                });
            *remaining -= ctx.input(|i| i.stable_dt);
            if *remaining <= 0.0 {
                self.toast = None;
            }
            ctx.request_repaint();
        }

        if matches!(self.screen, Screen::Downloading { .. }) {
            ctx.request_repaint();
        }
    }
}

impl App {
    fn ui_list(&mut self, ctx: &egui::Context, alpha: f32, offset_y: f32) {
        let t = self.lang.strings();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_opacity(alpha);
            ui.add_space(offset_y);

            ui.add_space(6.0);
            ui.label(RichText::new(t.pick_launcher).color(Color32::from_gray(190)));
            ui.add_space(10.0);

            let mut clicked_idx = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, entry) in LAUNCHERS.iter().enumerate() {
                    let resp = egui::Frame::none()
                        .fill(Color32::from_rgb(14, 14, 14))
                        .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 45, 45)))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(egui::Margin::symmetric(16.0, 14.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(entry.name).size(17.0).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let tags = match (entry.installer.is_some(), entry.portable.is_some()) {
                                        (true, true) => t.tag_both,
                                        (true, false) => t.tag_installer,
                                        (false, true) => t.tag_portable,
                                        (false, false) => t.tag_none,
                                    };
                                    ui.label(RichText::new(tags).color(Color32::from_gray(150)));
                                });
                            });
                        })
                        .response
                        .interact(egui::Sense::click());

                    if resp.hovered() {
                        ui.painter().rect_stroke(
                            resp.rect,
                            Rounding::same(10.0),
                            egui::Stroke::new(1.0_f32, Color32::from_rgb(72, 191, 116)),
                        );
                        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        clicked_idx = Some(idx);
                    }
                    ui.add_space(8.0);
                }
            });

            if let Some(idx) = clicked_idx {
                self.set_screen(Screen::Detail(idx));
            }
        });
    }

    fn ui_detail(&mut self, ctx: &egui::Context, idx: usize, alpha: f32, offset_y: f32) {
        let t = self.lang.strings();
        let entry = &LAUNCHERS[idx];
        let name = entry.name.to_string();
        let installer_url = entry.installer.map(|s| s.to_string());
        let portable_url = entry.portable.map(|s| s.to_string());
        let installer_filename = entry.installer_filename;

        let mut requested: Option<(VariantKind, String)> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_opacity(alpha);
            ui.add_space(offset_y);

            ui.add_space(10.0);
            ui.label(RichText::new(&name).size(24.0).strong());
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(16.0, 0.0);

                if let Some(url) = variant_card(ui, t.installer_label, installer_url.as_deref(), t) {
                    requested = Some((VariantKind::Installer, url));
                }
                if let Some(url) = variant_card(ui, t.portable_label, portable_url.as_deref(), t) {
                    requested = Some((VariantKind::Portable, url));
                }
            });

            ui.add_space(20.0);
            ui.label(
                RichText::new(format!("{}{}", t.save_location, self.save_dir.display()))
                    .color(Color32::from_gray(150))
                    .size(12.0),
            );
        });

        if let Some((kind, url)) = requested {
            let filename_override = match kind {
                VariantKind::Installer => installer_filename,
                VariantKind::Portable => None,
            };
            self.start_download(&name, kind, &url, filename_override);
        }
    }

    fn ui_downloading(&mut self, ctx: &egui::Context, alpha: f32, offset_y: f32) {
        let t = self.lang.strings();
        let (launcher, variant, progress, save_path) = match &self.screen {
            Screen::Downloading { launcher, variant, progress, save_path } => {
                (launcher.clone(), *variant, progress.clone(), save_path.clone())
            }
            _ => return,
        };

        let mut go_back = false;
        let mut launch_result: Option<Result<(), String>> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_opacity(alpha);
            ui.add_space(offset_y);

            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!("{}{launcher} ({})", t.downloading, variant.label(t)))
                        .size(20.0)
                        .strong(),
                );
                ui.add_space(20.0);

                let frac = progress.fraction();
                let downloaded = progress.downloaded.load(Ordering::Relaxed);
                let total = progress.total.load(Ordering::Relaxed);

                ui.add(
                    egui::ProgressBar::new(frac)
                        .desired_width(420.0)
                        .desired_height(18.0)
                        .fill(Color32::from_rgb(72, 191, 116))
                        .text(format!("{:.1}%", frac * 100.0)),
                );
                ui.add_space(10.0);

                if progress.merging.load(Ordering::Relaxed) {
                    ui.label(t.merging);
                } else if total > 0 {
                    ui.label(format!("{} / {}", format_bytes(downloaded), format_bytes(total)));
                } else {
                    ui.label(t.fetching_info);
                }

                if progress.done.load(Ordering::Relaxed) {
                    ui.add_space(16.0);
                    let err = progress.error.lock().unwrap().clone();
                    if let Some(err) = err {
                        ui.colored_label(
                            Color32::from_rgb(230, 100, 100),
                            format!("{}{err}", t.download_failed),
                        );
                    } else {
                        ui.colored_label(Color32::from_rgb(120, 220, 150), t.download_complete);

                        // Auto-launch the installer once download finishes successfully —
                        // skipped for Portable, which doesn't need an install step.
                        // `launched` guards this so it only ever fires once per download.
                        if variant == VariantKind::Installer
                            && !progress.launched.swap(true, Ordering::SeqCst)
                        {
                            launch_result = Some(open::that(&save_path).map_err(|e| e.to_string()));
                        }
                    }
                    ui.add_space(10.0);
                    if ui.button(t.back_to_list).clicked() {
                        go_back = true;
                    }
                }
            });
        });

        if let Some(result) = launch_result {
            match result {
                Ok(()) => self.show_toast(t.installer_launched),
                Err(e) => self.show_toast(format!("{}{e}", t.installer_launch_failed)),
            }
        }

        if go_back {
            self.set_screen(Screen::List);
        }
    }
}

/// Draws a single version (Installer / Portable) card; returns its URL if clicked.
fn variant_card(ui: &mut egui::Ui, label: &str, url: Option<&str>, t: &Strings) -> Option<String> {
    let enabled = url.is_some();
    let mut clicked_url = None;

    ui.add_enabled_ui(enabled, |ui| {
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 14, 14))
            .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 45, 45)))
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(24.0, 20.0))
            .show(ui, |ui| {
                ui.set_width(220.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(label).size(16.0).strong());
                    ui.add_space(10.0);
                    let btn_label = if enabled { t.download_btn } else { t.not_available };
                    if ui
                        .add_enabled(enabled, egui::Button::new(btn_label))
                        .clicked()
                    {
                        clicked_url = url.map(|u| u.to_string());
                    }
                });
            });
    });

    clicked_url
}

fn format_bytes(b: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}
