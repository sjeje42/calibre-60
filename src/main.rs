#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod alarm;
mod app;
mod clock;
mod dial;
mod i18n;
mod laps;
mod notifications;
mod settings;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod tray;

use app::Calibre60;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("fr.jeromelab.calibre60")
            .with_inner_size([620.0, 900.0])
            .with_min_inner_size([440.0, 620.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "Calibre 60 — JérômeLab",
        options,
        Box::new(|cc| Ok(Box::new(Calibre60::new(cc)))),
    )
}
