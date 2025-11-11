use eframe::egui;
use multi_boot::{device, disk, bootloader, iso, Result};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use super::ui::{DeviceView, IsoView, StatusView};

#[derive(Debug, Clone)]
pub enum AppMessage {
    DevicesLoaded(Vec<device::DeviceInfo>),
    DeviceSelected(String),
    InitializeStarted,
    InitializeProgress(String),
    InitializeCompleted(Result<()>),
    IsosLoaded(Vec<iso::IsoInfo>),
    AddIsoStarted(String),
    AddIsoProgress(f32, String),
    AddIsoCompleted(Result<()>),
    RemoveIsoCompleted(Result<()>),
    Error(String),
}

#[derive(Default)]
pub struct MultiBootApp {
    // State
    devices: Vec<device::DeviceInfo>,
    selected_device: Option<String>,
    isos: Vec<iso::IsoInfo>,

    // UI State
    device_filter_removable: bool,
    uefi_only: bool,
    volume_label: String,
    status_message: String,
    is_root: bool,

    // Operation state
    operation_in_progress: bool,
    operation_progress: f32,
    operation_status: String,

    // Communication
    tx: Option<Sender<AppMessage>>,
    rx: Option<Receiver<AppMessage>>,
}

impl MultiBootApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();

        let mut app = Self {
            devices: Vec::new(),
            selected_device: None,
            isos: Vec::new(),
            device_filter_removable: true,
            uefi_only: false,
            volume_label: "MULTIBOOT".to_string(),
            status_message: "Ready".to_string(),
            is_root: device::check_root().is_ok(),
            operation_in_progress: false,
            operation_progress: 0.0,
            operation_status: String::new(),
            tx: Some(tx.clone()),
            rx: Some(rx),
        };

        // Load devices on startup
        app.load_devices();

        app
    }

    fn load_devices(&mut self) {
        if let Some(tx) = &self.tx {
            let tx = tx.clone();
            thread::spawn(move || {
                match device::list_devices() {
                    Ok(devices) => {
                        let _ = tx.send(AppMessage::DevicesLoaded(devices));
                    }
                    Err(e) => {
                        let _ = tx.send(AppMessage::Error(format!("Failed to list devices: {}", e)));
                    }
                }
            });
        }
    }

    fn load_isos(&mut self) {
        if let Some(device_path) = &self.selected_device {
            if let Some(tx) = &self.tx {
                let tx = tx.clone();
                let device_path = device_path.clone();

                thread::spawn(move || {
                    // Try to find and mount the data partition
                    match device::find_device(&device_path) {
                        Ok(device_info) => {
                            if let Some(data_partition) = device_info.partitions.iter()
                                .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
                            {
                                // Check if mounted
                                if let Some(ref mount) = data_partition.mounted {
                                    match iso::list_isos(mount, false) {
                                        Ok(isos) => {
                                            let _ = tx.send(AppMessage::IsosLoaded(isos));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(AppMessage::Error(format!("Failed to list ISOs: {}", e)));
                                        }
                                    }
                                } else {
                                    let _ = tx.send(AppMessage::Error(
                                        "Data partition not mounted. Initialize the device first.".to_string()
                                    ));
                                }
                            } else {
                                let _ = tx.send(AppMessage::Error(
                                    "No MULTIBOOT partition found. Initialize the device first.".to_string()
                                ));
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::Error(format!("Failed to find device: {}", e)));
                        }
                    }
                });
            }
        }
    }

    fn initialize_device(&mut self) {
        if !self.is_root {
            self.status_message = "Error: Root privileges required!".to_string();
            return;
        }

        if let Some(device_path) = &self.selected_device {
            if let Some(tx) = &self.tx {
                let tx = tx.clone();
                let device_path = device_path.clone();
                let label = self.volume_label.clone();
                let uefi_only = self.uefi_only;

                self.operation_in_progress = true;
                self.operation_progress = 0.0;
                self.operation_status = "Initializing device...".to_string();

                let _ = tx.send(AppMessage::InitializeStarted);

                thread::spawn(move || {
                    let result = (|| -> Result<()> {
                        // Verify device
                        let _ = tx.send(AppMessage::InitializeProgress("Verifying device...".to_string()));
                        let device_info = device::verify_device(&device_path)?;

                        // Unmount if needed
                        if device::is_device_mounted(&device_info.path)? {
                            let _ = tx.send(AppMessage::InitializeProgress("Unmounting device...".to_string()));
                            device::unmount_device(&device_info.path)?;
                        }

                        // Create disk layout
                        let layout = disk::DiskLayout::from_device(&device_info.path, uefi_only);

                        // Create partitions
                        let _ = tx.send(AppMessage::InitializeProgress("Creating partitions...".to_string()));
                        let scheme = if uefi_only {
                            disk::PartitionScheme::UefiOnly
                        } else {
                            disk::PartitionScheme::Hybrid
                        };
                        disk::create_partitions(&device_info.path, scheme)?;

                        // Format partitions
                        let _ = tx.send(AppMessage::InitializeProgress("Formatting partitions...".to_string()));
                        disk::format_partitions(&layout.esp_partition, &layout.data_partition, &label)?;

                        // Create mountpoints
                        layout.create_mountpoints()?;

                        // Mount partitions
                        let _ = tx.send(AppMessage::InitializeProgress("Mounting partitions...".to_string()));
                        disk::mount_partition(&layout.esp_partition, &layout.esp_mountpoint)?;
                        disk::mount_partition(&layout.data_partition, &layout.data_mountpoint)?;

                        // Install GRUB
                        let _ = tx.send(AppMessage::InitializeProgress("Installing GRUB...".to_string()));
                        bootloader::install_grub(&device_info.path, &layout.esp_mountpoint, uefi_only)?;

                        // Create initial GRUB configuration
                        let isos_dir = layout.data_mountpoint.join("isos");
                        std::fs::create_dir_all(&isos_dir)?;

                        let grub_cfg_path = layout.esp_mountpoint.join("boot/grub/grub.cfg");
                        bootloader::generate_grub_config(&isos_dir, &grub_cfg_path)?;

                        // Unmount partitions
                        let _ = tx.send(AppMessage::InitializeProgress("Unmounting...".to_string()));
                        disk::unmount_partition(&layout.esp_mountpoint)?;
                        disk::unmount_partition(&layout.data_mountpoint)?;

                        // Cleanup
                        layout.cleanup_mountpoints()?;

                        Ok(())
                    })();

                    let _ = tx.send(AppMessage::InitializeCompleted(result));
                });
            }
        }
    }

    fn add_iso(&mut self, iso_path: PathBuf) {
        if !self.is_root {
            self.status_message = "Error: Root privileges required!".to_string();
            return;
        }

        if let Some(device_path) = &self.selected_device {
            if let Some(tx) = &self.tx {
                let tx = tx.clone();
                let device_path = device_path.clone();
                let iso_name = iso_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown.iso")
                    .to_string();

                self.operation_in_progress = true;
                self.operation_progress = 0.0;
                self.operation_status = format!("Adding {}...", iso_name);

                let _ = tx.send(AppMessage::AddIsoStarted(iso_name.clone()));

                thread::spawn(move || {
                    let result = (|| -> Result<()> {
                        // Find device
                        let device_info = device::find_device(&device_path)?;

                        // Find data partition
                        let data_partition = device_info.partitions.iter()
                            .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
                            .ok_or_else(|| multi_boot::MultiBootError::Other(
                                "MULTIBOOT partition not found".to_string()
                            ))?;

                        // Mount if needed
                        let mountpoint = if let Some(ref mount) = data_partition.mounted {
                            mount.clone()
                        } else {
                            let temp_mount = PathBuf::from("/tmp/multiboot-add");
                            std::fs::create_dir_all(&temp_mount)?;
                            disk::mount_partition(&data_partition.path, &temp_mount)?;
                            temp_mount
                        };

                        // Add ISO
                        let _ = tx.send(AppMessage::AddIsoProgress(0.5, "Copying ISO...".to_string()));
                        iso::add_iso(&mountpoint, &iso_path, None)?;

                        // Find ESP partition
                        let esp_partition = device_info.partitions.iter()
                            .find(|p| p.label.as_deref() == Some("ESP") ||
                                     (p.filesystem.as_deref() == Some("vfat") && p.number == 1))
                            .ok_or_else(|| multi_boot::MultiBootError::Other(
                                "ESP partition not found".to_string()
                            ))?;

                        // Mount ESP if needed
                        let esp_mountpoint = if let Some(ref mount) = esp_partition.mounted {
                            mount.clone()
                        } else {
                            let temp_mount = PathBuf::from("/tmp/multiboot-esp");
                            std::fs::create_dir_all(&temp_mount)?;
                            disk::mount_partition(&esp_partition.path, &temp_mount)?;
                            temp_mount
                        };

                        // Regenerate GRUB config
                        let _ = tx.send(AppMessage::AddIsoProgress(0.9, "Updating boot config...".to_string()));
                        let isos_dir = mountpoint.join("isos");
                        let grub_cfg_path = esp_mountpoint.join("boot/grub/grub.cfg");
                        bootloader::generate_grub_config(&isos_dir, &grub_cfg_path)?;

                        // Unmount if we mounted
                        if data_partition.mounted.is_none() {
                            disk::unmount_partition(&mountpoint)?;
                            let _ = std::fs::remove_dir(&mountpoint);
                        }

                        if esp_partition.mounted.is_none() {
                            disk::unmount_partition(&esp_mountpoint)?;
                            let _ = std::fs::remove_dir(&esp_mountpoint);
                        }

                        Ok(())
                    })();

                    let _ = tx.send(AppMessage::AddIsoCompleted(result));
                });
            }
        }
    }

    fn remove_iso(&mut self, iso_name: String) {
        if !self.is_root {
            self.status_message = "Error: Root privileges required!".to_string();
            return;
        }

        if let Some(device_path) = &self.selected_device {
            if let Some(tx) = &self.tx {
                let tx = tx.clone();
                let device_path = device_path.clone();

                self.operation_in_progress = true;
                self.operation_status = format!("Removing {}...", iso_name);

                thread::spawn(move || {
                    let result = (|| -> Result<()> {
                        // Find device
                        let device_info = device::find_device(&device_path)?;

                        // Find data partition
                        let data_partition = device_info.partitions.iter()
                            .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
                            .ok_or_else(|| multi_boot::MultiBootError::Other(
                                "MULTIBOOT partition not found".to_string()
                            ))?;

                        // Mount if needed
                        let mountpoint = if let Some(ref mount) = data_partition.mounted {
                            mount.clone()
                        } else {
                            let temp_mount = PathBuf::from("/tmp/multiboot-remove");
                            std::fs::create_dir_all(&temp_mount)?;
                            disk::mount_partition(&data_partition.path, &temp_mount)?;
                            temp_mount
                        };

                        // Remove ISO
                        iso::remove_iso(&mountpoint, &iso_name)?;

                        // Find ESP and regenerate config (similar to add_iso)
                        // ... (similar code as in add_iso for ESP mounting and config regen)

                        Ok(())
                    })();

                    let _ = tx.send(AppMessage::RemoveIsoCompleted(result));
                });
            }
        }
    }

    fn process_messages(&mut self, ctx: &egui::Context) {
        // Collect all messages first to avoid borrow checker issues
        let mut messages = Vec::new();
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        // Process messages
        for msg in messages {
            match msg {
                    AppMessage::DevicesLoaded(devices) => {
                        self.devices = devices;
                        self.status_message = format!("Found {} device(s)", self.devices.len());
                    }
                    AppMessage::DeviceSelected(path) => {
                        self.selected_device = Some(path);
                        self.load_isos();
                    }
                    AppMessage::InitializeStarted => {
                        self.operation_in_progress = true;
                        self.operation_status = "Initializing...".to_string();
                    }
                    AppMessage::InitializeProgress(msg) => {
                        self.operation_status = msg;
                        ctx.request_repaint();
                    }
                    AppMessage::InitializeCompleted(result) => {
                        self.operation_in_progress = false;
                        match result {
                            Ok(_) => {
                                self.status_message = "Device initialized successfully!".to_string();
                                self.load_isos();
                            }
                            Err(e) => {
                                self.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                    AppMessage::IsosLoaded(isos) => {
                        self.isos = isos;
                        self.status_message = format!("Found {} ISO(s)", self.isos.len());
                    }
                    AppMessage::AddIsoStarted(name) => {
                        self.operation_in_progress = true;
                        self.operation_status = format!("Adding {}...", name);
                    }
                    AppMessage::AddIsoProgress(progress, msg) => {
                        self.operation_progress = progress;
                        self.operation_status = msg;
                        ctx.request_repaint();
                    }
                    AppMessage::AddIsoCompleted(result) => {
                        self.operation_in_progress = false;
                        match result {
                            Ok(_) => {
                                self.status_message = "ISO added successfully!".to_string();
                                self.load_isos();
                            }
                            Err(e) => {
                                self.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                    AppMessage::RemoveIsoCompleted(result) => {
                        self.operation_in_progress = false;
                        match result {
                            Ok(_) => {
                                self.status_message = "ISO removed successfully!".to_string();
                                self.load_isos();
                            }
                            Err(e) => {
                                self.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                    AppMessage::Error(msg) => {
                        self.status_message = format!("Error: {}", msg);
                        self.operation_in_progress = false;
                    }
            }
        }
    }
}

pub enum IsoAction {
    Add(PathBuf),
    Remove(String),
}

impl eframe::App for MultiBootApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any incoming messages
        self.process_messages(ctx);

        // Top panel with title
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🖥️ Multi-Boot USB Creator");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !self.is_root {
                        ui.colored_label(egui::Color32::RED, "⚠️ Run with sudo for full functionality");
                    }
                });
            });
        });

        // Bottom panel with status
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);

                if self.operation_in_progress {
                    ui.spinner();
                    ui.label(&self.operation_status);
                }
            });
        });

        // Central panel with main content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Device selection section
                ui.group(|ui| {
                    ui.heading("1. Select USB Device");
                    ui.add_space(10.0);

                    if let Some(_newly_selected) = DeviceView::show(
                        ui,
                        &self.devices,
                        &mut self.selected_device,
                        &mut self.device_filter_removable,
                    ) {
                        self.load_isos();
                    }

                    ui.add_space(10.0);
                    if ui.button("🔄 Refresh Devices").clicked() {
                        self.load_devices();
                    }
                });

                ui.add_space(20.0);

                // Initialize device section
                if self.selected_device.is_some() {
                    ui.group(|ui| {
                        ui.heading("2. Initialize Device");
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label("Volume Label:");
                            ui.text_edit_singleline(&mut self.volume_label);
                        });

                        ui.checkbox(&mut self.uefi_only, "UEFI only (skip BIOS support)");

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("⚙️ Initialize Device").clicked() && !self.operation_in_progress {
                                self.initialize_device();
                            }

                            ui.colored_label(
                                egui::Color32::RED,
                                "⚠️ WARNING: This will ERASE ALL DATA on the device!",
                            );
                        });
                    });

                    ui.add_space(20.0);
                }

                // ISO management section
                if self.selected_device.is_some() && !self.isos.is_empty() || self.selected_device.is_some() {
                    ui.group(|ui| {
                        ui.heading("3. Manage ISOs");
                        ui.add_space(10.0);

                        if let Some(action) = IsoView::show(ui, &self.isos, self.is_root) {
                            match action {
                                IsoAction::Add(path) => self.add_iso(path),
                                IsoAction::Remove(name) => self.remove_iso(name),
                            }
                        }
                    });
                }

                // Progress section
                if self.operation_in_progress && self.operation_progress > 0.0 {
                    ui.add_space(20.0);
                    ui.group(|ui| {
                        ui.heading("Progress");
                        ui.add_space(10.0);

                        let progress_bar = egui::ProgressBar::new(self.operation_progress)
                            .text(&self.operation_status);
                        ui.add(progress_bar);
                    });
                }
            });
        });
    }
}
