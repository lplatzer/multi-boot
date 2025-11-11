use crate::{Result, MultiBootError};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum PartitionScheme {
    /// UEFI + BIOS hybrid boot
    Hybrid,
    /// UEFI only
    UefiOnly,
}

/// Create partition table and partitions on the device
pub fn create_partitions<P: AsRef<Path>>(
    device: P,
    scheme: PartitionScheme,
) -> Result<()> {
    let device_path = device.as_ref();

    println!("Creating partition table on {}...", device_path.display());

    // Step 1: Wipe existing partition table
    wipe_partition_table(device_path)?;

    // Step 2: Create GPT partition table
    create_gpt_table(device_path)?;

    // Step 3: Create partitions based on scheme
    match scheme {
        PartitionScheme::Hybrid => create_hybrid_partitions(device_path)?,
        PartitionScheme::UefiOnly => create_uefi_partitions(device_path)?,
    }

    // Step 4: Set partition flags
    set_partition_flags(device_path, &scheme)?;

    // Step 5: Inform kernel of partition changes
    reload_partition_table(device_path)?;

    println!("Partitions created successfully");
    Ok(())
}

fn wipe_partition_table<P: AsRef<Path>>(device: P) -> Result<()> {
    let device_path = device.as_ref();

    // Use wipefs to remove signatures
    let output = Command::new("wipefs")
        .args(&["--all", device_path.to_str().unwrap()])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "wipefs".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        eprintln!("Warning: wipefs failed: {}",
            String::from_utf8_lossy(&output.stderr));
    }

    // Zero out first few MB
    let output = Command::new("dd")
        .args(&[
            "if=/dev/zero",
            &format!("of={}", device_path.display()),
            "bs=1M",
            "count=10",
            "status=none",
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "dd".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::PartitionError(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(())
}

fn create_gpt_table<P: AsRef<Path>>(device: P) -> Result<()> {
    let device_path = device.as_ref();

    let output = Command::new("parted")
        .args(&[
            "-s",
            device_path.to_str().unwrap(),
            "mklabel",
            "gpt",
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "parted".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::PartitionError(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(())
}

fn create_hybrid_partitions<P: AsRef<Path>>(device: P) -> Result<()> {
    let device_path = device.as_ref();

    // Partition 1: EFI System Partition (512MB)
    run_parted(device_path, &[
        "mkpart", "primary", "fat32", "1MiB", "513MiB"
    ])?;

    // Partition 2: BIOS Boot Partition (1MB)
    run_parted(device_path, &[
        "mkpart", "primary", "513MiB", "514MiB"
    ])?;

    // Partition 3: Data Partition (remaining space)
    run_parted(device_path, &[
        "mkpart", "primary", "exfat", "514MiB", "100%"
    ])?;

    Ok(())
}

fn create_uefi_partitions<P: AsRef<Path>>(device: P) -> Result<()> {
    let device_path = device.as_ref();

    // Partition 1: EFI System Partition (512MB)
    run_parted(device_path, &[
        "mkpart", "primary", "fat32", "1MiB", "513MiB"
    ])?;

    // Partition 2: Data Partition (remaining space)
    run_parted(device_path, &[
        "mkpart", "primary", "exfat", "513MiB", "100%"
    ])?;

    Ok(())
}

fn set_partition_flags<P: AsRef<Path>>(device: P, scheme: &PartitionScheme) -> Result<()> {
    let device_path = device.as_ref();

    // Set ESP flag on partition 1
    run_parted(device_path, &["set", "1", "esp", "on"])?;

    // Set BIOS boot flag on partition 2 (hybrid mode only)
    if matches!(scheme, PartitionScheme::Hybrid) {
        run_parted(device_path, &["set", "2", "bios_grub", "on"])?;
    }

    Ok(())
}

fn reload_partition_table<P: AsRef<Path>>(device: P) -> Result<()> {
    let device_path = device.as_ref();

    // Try partprobe first
    let output = Command::new("partprobe")
        .arg(device_path)
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "partprobe".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        eprintln!("Warning: partprobe failed, trying blockdev...");

        // Fallback to blockdev --rereadpt
        let output = Command::new("blockdev")
            .args(&["--rereadpt", device_path.to_str().unwrap()])
            .output()
            .map_err(|e| MultiBootError::CommandFailed {
                cmd: "blockdev".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            eprintln!("Warning: blockdev failed. You may need to replug the device.");
        }
    }

    // Wait for device nodes to appear
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

fn run_parted<P: AsRef<Path>>(device: P, args: &[&str]) -> Result<()> {
    let device_path = device.as_ref();

    let mut cmd_args = vec!["-s", device_path.to_str().unwrap()];
    cmd_args.extend_from_slice(args);

    let output = Command::new("parted")
        .args(&cmd_args)
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "parted".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::PartitionError(
            format!("parted failed: {}", String::from_utf8_lossy(&output.stderr))
        ));
    }

    Ok(())
}
