mod app;
mod ui;

use app::MultiBootApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Multi-Boot USB Creator",
        options,
        Box::new(|cc| Box::new(MultiBootApp::new(cc))),
    )
}
