use crate::{Result, MultiBootError};
use std::path::Path;
use std::process::Command;

/// Install GRUB bootloader for both BIOS and UEFI
pub fn install_grub<P: AsRef<Path>>(
    device: P,
    esp_mountpoint: P,
    uefi_only: bool,
) -> Result<()> {
    println!("Installing GRUB bootloader...");

    // Install GRUB for UEFI
    install_grub_uefi(&esp_mountpoint)?;

    // Install GRUB for BIOS (if not UEFI-only)
    if !uefi_only {
        install_grub_bios(&device, &esp_mountpoint)?;
    }

    println!("GRUB installed successfully");
    Ok(())
}

fn install_grub_uefi<P: AsRef<Path>>(esp_mountpoint: P) -> Result<()> {
    let mountpoint = esp_mountpoint.as_ref();

    println!("  Installing GRUB for UEFI...");

    // Create EFI directory structure
    let efi_dir = mountpoint.join("EFI/BOOT");
    std::fs::create_dir_all(&efi_dir)?;

    // Install GRUB EFI bootloader
    let output = Command::new("grub-install")
        .args(&[
            "--target=x86_64-efi",
            "--efi-directory", mountpoint.to_str().unwrap(),
            "--boot-directory", mountpoint.join("boot").to_str().unwrap(),
            "--bootloader-id=BOOT",
            "--removable",  // Use fallback path /EFI/BOOT/BOOTX64.EFI
            "--no-nvram",   // Don't modify UEFI boot variables
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "grub-install".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::BootloaderError(
            format!("UEFI GRUB installation failed: {}",
                String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}

fn install_grub_bios<P: AsRef<Path>>(device: P, esp_mountpoint: P) -> Result<()> {
    let device_path = device.as_ref();
    let mountpoint = esp_mountpoint.as_ref();

    println!("  Installing GRUB for BIOS...");

    // Install GRUB for BIOS/Legacy boot
    let output = Command::new("grub-install")
        .args(&[
            "--target=i386-pc",
            "--boot-directory", mountpoint.join("boot").to_str().unwrap(),
            device_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "grub-install".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::BootloaderError(
            format!("BIOS GRUB installation failed: {}",
                String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}

/// Check if GRUB tools are available
pub fn check_grub_tools() -> Result<Vec<String>> {
    let mut missing = Vec::new();

    if !command_exists("grub-install") {
        missing.push("grub-install (install grub2 or grub-pc-bin)".to_string());
    }

    // Check for GRUB modules
    if !grub_modules_exist("i386-pc") {
        missing.push("GRUB BIOS modules (install grub-pc-bin)".to_string());
    }

    if !grub_modules_exist("x86_64-efi") {
        missing.push("GRUB UEFI modules (install grub-efi-amd64-bin)".to_string());
    }

    Ok(missing)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn grub_modules_exist(target: &str) -> bool {
    // Common GRUB module directories
    let grub_dirs = vec![
        format!("/usr/lib/grub/{}", target),
        format!("/usr/share/grub/{}", target),
        format!("/boot/grub/{}", target),
    ];

    grub_dirs.iter().any(|dir| {
        std::path::Path::new(dir).exists()
    })
}
