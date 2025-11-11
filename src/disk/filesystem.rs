use crate::{Result, MultiBootError};
use std::path::Path;
use std::process::Command;

/// Format partitions with appropriate filesystems
pub fn format_partitions<P: AsRef<Path>>(
    esp_partition: P,
    data_partition: P,
    label: &str,
) -> Result<()> {
    println!("Formatting partitions...");

    // Format ESP as FAT32
    format_fat32(&esp_partition, "ESP")?;

    // Format data partition as exFAT
    format_exfat(&data_partition, label)?;

    println!("Partitions formatted successfully");
    Ok(())
}

fn format_fat32<P: AsRef<Path>>(partition: P, label: &str) -> Result<()> {
    let partition_path = partition.as_ref();

    println!("  Formatting {} as FAT32 (label: {})...",
        partition_path.display(), label);

    // Use mkfs.vfat (from dosfstools)
    let output = Command::new("mkfs.vfat")
        .args(&[
            "-F", "32",                    // FAT32
            "-n", label,                   // Volume label
            partition_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "mkfs.vfat".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::FilesystemError(
            format!("Failed to format FAT32: {}",
                String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}

fn format_exfat<P: AsRef<Path>>(partition: P, label: &str) -> Result<()> {
    let partition_path = partition.as_ref();

    println!("  Formatting {} as exFAT (label: {})...",
        partition_path.display(), label);

    // Use mkfs.exfat (from exfatprogs or exfat-utils)
    let output = Command::new("mkfs.exfat")
        .args(&[
            "-n", label,                   // Volume label
            partition_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "mkfs.exfat".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::FilesystemError(
            format!("Failed to format exFAT: {}",
                String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}

/// Mount a partition to a mountpoint
pub fn mount_partition<P1: AsRef<Path>, P2: AsRef<Path>>(
    partition: P1,
    mountpoint: P2,
) -> Result<()> {
    let partition_path = partition.as_ref();
    let mountpoint_path = mountpoint.as_ref();

    // Create mountpoint if it doesn't exist
    std::fs::create_dir_all(mountpoint_path)?;

    let output = Command::new("mount")
        .args(&[
            partition_path.to_str().unwrap(),
            mountpoint_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "mount".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::FilesystemError(
            format!("Failed to mount {}: {}",
                partition_path.display(),
                String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}

/// Unmount a partition
pub fn unmount_partition<P: AsRef<Path>>(mountpoint: P) -> Result<()> {
    let mountpoint_path = mountpoint.as_ref();

    let output = Command::new("umount")
        .arg(mountpoint_path)
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "umount".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::FilesystemError(
            format!("Failed to unmount {}: {}",
                mountpoint_path.display(),
                String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}

/// Check if required filesystem tools are available
pub fn check_filesystem_tools() -> Result<Vec<String>> {
    let mut missing = Vec::new();

    let tools = vec![
        ("mkfs.vfat", "dosfstools"),
        ("mkfs.exfat", "exfatprogs or exfat-utils"),
    ];

    for (cmd, package) in tools {
        if !command_exists(cmd) {
            missing.push(format!("{} (install {})", cmd, package));
        }
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
