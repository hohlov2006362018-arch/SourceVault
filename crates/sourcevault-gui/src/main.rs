//! SourceVault GUI entry point.
//!
//! The Windows build is tagged `#![windows_subsystem = "windows"]` so the executable does not
//! summon a console window when launched from Explorer. The CLI binary is a separate crate.

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod i18n;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("SourceVault")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "SourceVault",
        native_options,
        Box::new(|cc| Box::new(app::SourceVaultApp::new(cc))),
    )
}

fn load_icon() -> egui::IconData {
    // 32x32 simple lambda-shaped icon as a fallback. Real ICO is bundled via winres at build time
    // on Windows; this fallback ensures cross-platform builds still get a window icon.
    let size: usize = 32;
    let mut pixels = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let fx = x as f32 / size as f32 - 0.5;
            let fy = y as f32 / size as f32 - 0.5;
            let r2 = fx * fx + fy * fy;
            if r2 < 0.2 {
                pixels[idx] = 0xF6;
                pixels[idx + 1] = 0x8E;
                pixels[idx + 2] = 0x1F;
                pixels[idx + 3] = 0xFF;
            } else {
                pixels[idx + 3] = 0x00;
            }
        }
    }
    egui::IconData {
        rgba: pixels,
        width: size as u32,
        height: size as u32,
    }
}
