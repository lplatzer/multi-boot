use eframe::egui;
use multi_boot::device::DeviceInfo;

pub struct DeviceView;

impl DeviceView {
    pub fn show(
        ui: &mut egui::Ui,
        devices: &[DeviceInfo],
        selected_device: &mut Option<String>,
        filter_removable: &mut bool,
    ) -> Option<String> {
        ui.checkbox(filter_removable, "Show only removable devices");
        ui.add_space(10.0);

        if devices.is_empty() {
            ui.label("No devices found. Click 'Refresh Devices' to scan.");
            return None;
        }

        let filtered_devices: Vec<_> = devices
            .iter()
            .filter(|d| !*filter_removable || d.removable)
            .collect();

        if filtered_devices.is_empty() {
            ui.label("No removable devices found.");
            return None;
        }

        let mut newly_selected = None;

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for device in filtered_devices {
                    let is_selected = selected_device
                        .as_ref()
                        .map(|s| s == &device.path.to_string_lossy())
                        .unwrap_or(false);

                    let response = ui.selectable_label(
                        is_selected,
                        format!(
                            "{} - {} {}{}",
                            device.path.display(),
                            device.size_human,
                            device.model.as_ref().map(|m| format!("({})", m)).unwrap_or_default(),
                            if device.removable { " 📀" } else { "" }
                        ),
                    );

                    if response.clicked() {
                        let path = device.path.to_string_lossy().to_string();
                        *selected_device = Some(path.clone());
                        newly_selected = Some(path);
                    }

                    // Show tooltip with more info
                    response.on_hover_text(format!(
                        "Path: {}\nSize: {} ({} bytes)\n{}Removable: {}",
                        device.path.display(),
                        device.size_human,
                        device.size,
                        device.vendor.as_ref().map(|v| format!("Vendor: {}\n", v)).unwrap_or_default(),
                        if device.removable { "Yes" } else { "No" }
                    ));
                }
            });

        newly_selected
    }
}
