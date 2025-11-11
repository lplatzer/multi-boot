use crate::{Result, MultiBootError};
use super::info::DeviceInfo;
use std::path::{Path, PathBuf};
use std::process::Command;

/// List all block devices on the system
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let output = Command::new("lsblk")
        .args(&[
            "-b", // bytes
            "-d", // devices only (no partitions)
            "-n", // no headers
            "-o", "NAME,SIZE,TYPE,HOTPLUG,VENDOR,MODEL",
            "-p", // full paths
        ])
        .output()
        .map_err(|e| MultiBootError::CommandFailed {
            cmd: "lsblk".to_string(),
            message: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(MultiBootError::CommandFailed {
            cmd: "lsblk".to_string(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let device_path = parts[0];

        // Skip loop devices and other non-physical devices
        if device_path.contains("/loop") || device_path.contains("/dm-") {
            continue;
        }

        // Get full device info
        if let Ok(info) = DeviceInfo::from_device(device_path) {
            devices.push(info);
        }
    }

    Ok(devices)
}

/// List only removable USB devices
pub fn list_usb_devices() -> Result<Vec<DeviceInfo>> {
    let all_devices = list_devices()?;
    Ok(all_devices.into_iter()
        .filter(|d| d.removable)
        .collect())
}

/// Find a device by path or name
pub fn find_device<P: AsRef<Path>>(device: P) -> Result<DeviceInfo> {
    let device_path = device.as_ref();

    // If it's a full path, use it directly
    if device_path.starts_with("/dev") {
        return DeviceInfo::from_device(device_path);
    }

    // Otherwise, try to find it in /dev
    let full_path = PathBuf::from("/dev").join(device_path);
    DeviceInfo::from_device(full_path)
}

/// Check if a device is safe to modify (not a system disk)
pub fn is_safe_device<P: AsRef<Path>>(device: P) -> Result<bool> {
    let info = DeviceInfo::from_device(device)?;

    // Not safe if not removable
    if !info.removable {
        return Ok(false);
    }

    // Not safe if any partition is mounted on system paths
    for partition in &info.partitions {
        if let Some(ref mount) = partition.mounted {
            let mount_str = mount.to_string_lossy();
            // Check for system mount points
            if mount_str == "/" ||
               mount_str.starts_with("/boot") ||
               mount_str.starts_with("/home") ||
               mount_str.starts_with("/usr") ||
               mount_str.starts_with("/var") {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Verify device is removable and safe to use
pub fn verify_device<P: AsRef<Path>>(device: P) -> Result<DeviceInfo> {
    let info = DeviceInfo::from_device(&device)?;

    if !info.removable {
        return Err(MultiBootError::NotRemovable(
            info.path.display().to_string()
        ));
    }

    if !is_safe_device(&device)? {
        return Err(MultiBootError::Other(
            format!("Device {} appears to be a system disk", info.path.display())
        ));
    }

    Ok(info)
}
