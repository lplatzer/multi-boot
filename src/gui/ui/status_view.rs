use eframe::egui;

pub struct StatusView;

impl StatusView {
    pub fn show(ui: &mut egui::Ui, message: &str, in_progress: bool) {
        ui.horizontal(|ui| {
            if in_progress {
                ui.spinner();
            }
            ui.label(message);
        });
    }
}
