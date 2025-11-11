use clap::Parser;
use multi_boot::{
    cli::{Cli, Commands},
    device, disk, bootloader, iso,
    Result, MultiBootError,
};
use std::io::{self, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Devices { detailed } => cmd_devices(detailed),
        Commands::Init { device, label, uefi_only } => {
            cmd_init(&device, &label, uefi_only, cli.yes)
        }
        Commands::Add { device, iso, name } => {
            cmd_add(&device, &iso, name.as_deref())
        }
        Commands::List { device, detailed } => {
            cmd_list(&device, detailed)
        }
        Commands::Remove { device, iso } => {
            cmd_remove(&device, &iso, cli.yes)
        }
        Commands::Refresh { device } => {
            cmd_refresh(&device)
        }
        Commands::Check { fix } => {
            cmd_check(fix)
        }
    }
}

fn cmd_devices(detailed: bool) -> Result<()> {
    println!("Scanning for USB devices...\n");

    let devices = device::list_devices()?;
    let usb_devices: Vec<_> = devices.iter()
        .filter(|d| d.removable)
        .collect();

    if usb_devices.is_empty() {
        println!("No USB devices found.");
        return Ok(());
    }

    for dev in usb_devices {
        println!("Device: {}", dev.path.display());
        println!("  Name: {}", dev.name);
        println!("  Size: {} ({})", dev.size_human, dev.size);

        if let Some(ref vendor) = dev.vendor {
            println!("  Vendor: {}", vendor);
        }
        if let Some(ref model) = dev.model {
            println!("  Model: {}", model);
        }

        println!("  Removable: {}", if dev.removable { "Yes" } else { "No" });

        if detailed && !dev.partitions.is_empty() {
            println!("  Partitions:");
            for part in &dev.partitions {
                println!("    {} - {} bytes", part.path.display(), part.size);
                if let Some(ref fs) = part.filesystem {
                    println!("      Filesystem: {}", fs);
                }
                if let Some(ref label) = part.label {
                    println!("      Label: {}", label);
                }
                if let Some(ref mount) = part.mounted {
                    println!("      Mounted: {}", mount.display());
                }
            }
        }

        println!();
    }

    Ok(())
}

fn cmd_init(device_path: &str, label: &str, uefi_only: bool, auto_yes: bool) -> Result<()> {
    // Check root privileges
    device::check_root()?;

    // Validate and verify device
    let device_info = device::verify_device(device_path)?;

    println!("Initializing multi-boot USB drive");
    println!("================================");
    println!("Device: {}", device_info.path.display());
    println!("Size: {}", device_info.size_human);
    if let Some(ref model) = device_info.model {
        println!("Model: {}", model);
    }
    println!("Label: {}", label);
    println!("Mode: {}", if uefi_only { "UEFI only" } else { "UEFI + BIOS" });
    println!();
    println!("WARNING: This will ERASE ALL DATA on the device!");

    if !auto_yes && !confirm("Do you want to continue?")? {
        println!("Operation cancelled.");
        return Err(MultiBootError::Cancelled);
    }

    // Check if device is mounted and unmount
    if device::is_device_mounted(&device_info.path)? {
        println!("Unmounting device...");
        device::unmount_device(&device_info.path)?;
    }

    // Create disk layout
    let layout = disk::DiskLayout::from_device(&device_info.path, uefi_only);

    // Create partitions
    let scheme = if uefi_only {
        disk::PartitionScheme::UefiOnly
    } else {
        disk::PartitionScheme::Hybrid
    };
    disk::create_partitions(&device_info.path, scheme)?;

    // Format partitions
    disk::format_partitions(&layout.esp_partition, &layout.data_partition, label)?;

    // Create and mount temporary directories
    layout.create_mountpoints()?;

    // Mount partitions
    println!("Mounting partitions...");
    disk::mount_partition(&layout.esp_partition, &layout.esp_mountpoint)?;
    disk::mount_partition(&layout.data_partition, &layout.data_mountpoint)?;

    // Install GRUB
    bootloader::install_grub(&device_info.path, &layout.esp_mountpoint, uefi_only)?;

    // Create initial GRUB configuration
    let isos_dir = layout.data_mountpoint.join("isos");
    std::fs::create_dir_all(&isos_dir)?;

    let grub_cfg_path = layout.esp_mountpoint.join("boot/grub/grub.cfg");
    bootloader::generate_grub_config(&isos_dir, &grub_cfg_path)?;

    // Unmount partitions
    println!("Unmounting partitions...");
    disk::unmount_partition(&layout.esp_mountpoint)?;
    disk::unmount_partition(&layout.data_mountpoint)?;

    // Cleanup
    layout.cleanup_mountpoints()?;

    println!();
    println!("Success! Multi-boot USB drive is ready.");
    println!("You can now add ISOs using: multi-boot add {} <iso-file>", device_info.path.display());

    Ok(())
}

fn cmd_add(device_path: &str, iso_path: &std::path::Path, custom_name: Option<&str>) -> Result<()> {
    // Check root privileges
    device::check_root()?;

    // Find device
    let device_info = device::find_device(device_path)?;

    // Find data partition (should be labeled MULTIBOOT)
    let data_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
        .ok_or_else(|| MultiBootError::Other(
            "Could not find MULTIBOOT partition. Is the device initialized?".to_string()
        ))?;

    // Mount data partition if not already mounted
    let mountpoint = if let Some(ref mount) = data_partition.mounted {
        mount.clone()
    } else {
        let temp_mount = std::path::PathBuf::from("/tmp/multiboot-add");
        std::fs::create_dir_all(&temp_mount)?;
        disk::mount_partition(&data_partition.path, &temp_mount)?;
        temp_mount
    };

    // Add ISO
    iso::add_iso(&mountpoint, iso_path, custom_name)?;

    // Find ESP partition
    let esp_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("ESP") ||
                  p.filesystem.as_deref() == Some("vfat") &&
                  p.number == 1)
        .ok_or_else(|| MultiBootError::Other(
            "Could not find ESP partition".to_string()
        ))?;

    // Mount ESP if not mounted
    let esp_mountpoint = if let Some(ref mount) = esp_partition.mounted {
        mount.clone()
    } else {
        let temp_mount = std::path::PathBuf::from("/tmp/multiboot-esp");
        std::fs::create_dir_all(&temp_mount)?;
        disk::mount_partition(&esp_partition.path, &temp_mount)?;
        temp_mount
    };

    // Regenerate GRUB config
    println!("Updating boot configuration...");
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

    println!("Done! ISO added successfully.");

    Ok(())
}

fn cmd_list(device_path: &str, detailed: bool) -> Result<()> {
    // Find device
    let device_info = device::find_device(device_path)?;

    // Find data partition
    let data_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
        .ok_or_else(|| MultiBootError::Other(
            "Could not find MULTIBOOT partition. Is the device initialized?".to_string()
        ))?;

    // Mount data partition if not already mounted
    let mountpoint = if let Some(ref mount) = data_partition.mounted {
        mount.clone()
    } else {
        let temp_mount = std::path::PathBuf::from("/tmp/multiboot-list");
        std::fs::create_dir_all(&temp_mount)?;
        disk::mount_partition(&data_partition.path, &temp_mount)?;
        temp_mount
    };

    println!("ISOs on {}:\n", device_info.path.display());
    let isos = iso::list_isos(&mountpoint, detailed)?;

    if isos.is_empty() {
        println!("  No ISOs found.");
    }

    // Unmount if we mounted
    if data_partition.mounted.is_none() {
        disk::unmount_partition(&mountpoint)?;
        let _ = std::fs::remove_dir(&mountpoint);
    }

    Ok(())
}

fn cmd_remove(device_path: &str, iso_name: &str, auto_yes: bool) -> Result<()> {
    // Check root privileges
    device::check_root()?;

    if !auto_yes && !confirm(&format!("Remove ISO '{}'?", iso_name))? {
        println!("Operation cancelled.");
        return Err(MultiBootError::Cancelled);
    }

    // Find device
    let device_info = device::find_device(device_path)?;

    // Find data partition
    let data_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
        .ok_or_else(|| MultiBootError::Other(
            "Could not find MULTIBOOT partition. Is the device initialized?".to_string()
        ))?;

    // Mount data partition if not already mounted
    let mountpoint = if let Some(ref mount) = data_partition.mounted {
        mount.clone()
    } else {
        let temp_mount = std::path::PathBuf::from("/tmp/multiboot-remove");
        std::fs::create_dir_all(&temp_mount)?;
        disk::mount_partition(&data_partition.path, &temp_mount)?;
        temp_mount
    };

    // Remove ISO
    iso::remove_iso(&mountpoint, iso_name)?;

    // Find ESP partition
    let esp_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("ESP") ||
                  p.filesystem.as_deref() == Some("vfat") &&
                  p.number == 1)
        .ok_or_else(|| MultiBootError::Other(
            "Could not find ESP partition".to_string()
        ))?;

    // Mount ESP if not mounted
    let esp_mountpoint = if let Some(ref mount) = esp_partition.mounted {
        mount.clone()
    } else {
        let temp_mount = std::path::PathBuf::from("/tmp/multiboot-esp");
        std::fs::create_dir_all(&temp_mount)?;
        disk::mount_partition(&esp_partition.path, &temp_mount)?;
        temp_mount
    };

    // Regenerate GRUB config
    println!("Updating boot configuration...");
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

    println!("Done!");

    Ok(())
}

fn cmd_refresh(device_path: &str) -> Result<()> {
    // Check root privileges
    device::check_root()?;

    println!("Refreshing boot configuration...");

    // Find device
    let device_info = device::find_device(device_path)?;

    // Find partitions
    let data_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("MULTIBOOT"))
        .ok_or_else(|| MultiBootError::Other(
            "Could not find MULTIBOOT partition".to_string()
        ))?;

    let esp_partition = device_info.partitions.iter()
        .find(|p| p.label.as_deref() == Some("ESP") ||
                  p.filesystem.as_deref() == Some("vfat") &&
                  p.number == 1)
        .ok_or_else(|| MultiBootError::Other(
            "Could not find ESP partition".to_string()
        ))?;

    // Mount partitions
    let data_mount = std::path::PathBuf::from("/tmp/multiboot-refresh-data");
    let esp_mount = std::path::PathBuf::from("/tmp/multiboot-refresh-esp");

    std::fs::create_dir_all(&data_mount)?;
    std::fs::create_dir_all(&esp_mount)?;

    disk::mount_partition(&data_partition.path, &data_mount)?;
    disk::mount_partition(&esp_partition.path, &esp_mount)?;

    // Regenerate config
    let isos_dir = data_mount.join("isos");
    let grub_cfg_path = esp_mount.join("boot/grub/grub.cfg");
    bootloader::generate_grub_config(&isos_dir, &grub_cfg_path)?;

    // Unmount
    disk::unmount_partition(&data_mount)?;
    disk::unmount_partition(&esp_mount)?;

    let _ = std::fs::remove_dir(&data_mount);
    let _ = std::fs::remove_dir(&esp_mount);

    println!("Boot configuration refreshed!");

    Ok(())
}

fn cmd_check(_fix: bool) -> Result<()> {
    println!("Checking system requirements...\n");

    let mut all_ok = true;

    // Check root
    if device::check_root().is_err() {
        println!("[ ] Root privileges (run with sudo)");
        all_ok = false;
    } else {
        println!("[✓] Root privileges");
    }

    // Check partition tools
    if command_exists("parted") {
        println!("[✓] parted");
    } else {
        println!("[ ] parted (install parted package)");
        all_ok = false;
    }

    // Check filesystem tools
    let fs_missing = disk::check_filesystem_tools()?;
    if fs_missing.is_empty() {
        println!("[✓] Filesystem tools (mkfs.vfat, mkfs.exfat)");
    } else {
        for tool in &fs_missing {
            println!("[ ] {}", tool);
        }
        all_ok = false;
    }

    // Check bootloader tools
    let grub_missing = bootloader::check_bootloader_requirements()?;
    if grub_missing.is_empty() {
        println!("[✓] GRUB bootloader");
    } else {
        for tool in &grub_missing {
            println!("[ ] {}", tool);
        }
        all_ok = false;
    }

    // Check other utilities
    for cmd in &["wipefs", "lsblk", "mount", "umount"] {
        if command_exists(cmd) {
            println!("[✓] {}", cmd);
        } else {
            println!("[ ] {} (install util-linux package)", cmd);
            all_ok = false;
        }
    }

    println!();
    if all_ok {
        println!("All requirements satisfied!");
    } else {
        println!("Some requirements are missing. Please install the required packages.");
        return Err(MultiBootError::Other(
            "Missing system requirements".to_string()
        ));
    }

    Ok(())
}

fn confirm(message: &str) -> Result<bool> {
    print!("{} [y/N]: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
