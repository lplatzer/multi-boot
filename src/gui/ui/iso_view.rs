use eframe::egui;
use multi_boot::iso::IsoInfo;
use std::path::PathBuf;

use super::super::app::IsoAction;

pub struct IsoView;

impl IsoView {
    pub fn show(
        ui: &mut egui::Ui,
        isos: &[IsoInfo],
        is_root: bool,
    ) -> Option<IsoAction> {
        let mut action = None;

        // Add ISO button
        ui.horizontal(|ui| {
            if ui.button("➕ Add ISO").clicked() && is_root {
                // Use native file dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("ISO Image", &["iso"])
                    .pick_file()
                {
                    action = Some(IsoAction::Add(path));
                }
            }

            if !is_root {
                ui.colored_label(egui::Color32::YELLOW, "(Root privileges required)");
            }
        });

        ui.add_space(10.0);

        // List ISOs
        if isos.is_empty() {
            ui.label("No ISOs on this device. Add one using the button above.");
            return action;
        }

        ui.label(format!("ISOs on device ({}):", isos.len()));
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("iso_grid")
                    .num_columns(3)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Header
                        ui.label(egui::RichText::new("Filename").strong());
                        ui.label(egui::RichText::new("Size").strong());
                        ui.label(egui::RichText::new("Action").strong());
                        ui.end_row();

                        // ISO rows
                        for iso_info in isos {
                            ui.label(&iso_info.filename);
                            ui.label(&iso_info.size_human);

                            if is_root {
                                if ui.button("🗑️ Remove").clicked() {
                                    action = Some(IsoAction::Remove(iso_info.filename.clone()));
                                }
                            } else {
                                ui.label("—");
                            }

                            ui.end_row();
                        }
                    });
            });

        action
    }
}
