use crate::{Result, MultiBootError};
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub size_human: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub removable: bool,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub path: PathBuf,
    pub number: u32,
    pub size: u64,
    pub filesystem: Option<String>,
    pub label: Option<String>,
    pub mounted: Option<PathBuf>,
}

impl DeviceInfo {
    /// Get device information using lsblk
    pub fn from_device<P: AsRef<Path>>(device: P) -> Result<Self> {
        let device_path = device.as_ref();

        // Use lsblk to get device information
        let output = Command::new("lsblk")
            .args(&[
                "-b", // bytes
                "-P", // pairs
                "-o", "NAME,SIZE,TYPE,HOTPLUG,VENDOR,MODEL,FSTYPE,LABEL,MOUNTPOINT",
                device_path.to_str().unwrap(),
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
        Self::parse_lsblk(&stdout, device_path)
    }

    fn parse_lsblk(output: &str, device_path: &Path) -> Result<Self> {
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return Err(MultiBootError::DeviceNotFound(
                device_path.display().to_string()
            ));
        }

        // Parse device line (first line)
        let device_line = lines[0];
        let device_fields = parse_lsblk_line(device_line);

        let name = device_fields.get("NAME")
            .ok_or_else(|| MultiBootError::ParseError("Missing NAME field".to_string()))?
            .to_string();

        let size = device_fields.get("SIZE")
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| MultiBootError::ParseError("Invalid SIZE field".to_string()))?;

        let removable = device_fields.get("HOTPLUG")
            .map(|s| s == "1")
            .unwrap_or(false);

        // Parse partitions (remaining lines)
        let mut partitions = Vec::new();
        for line in &lines[1..] {
            let fields = parse_lsblk_line(line);
            if fields.get("TYPE").map(|s| s.as_str()) == Some("part") {
                if let Some(part_info) = parse_partition(&fields) {
                    partitions.push(part_info);
                }
            }
        }

        Ok(DeviceInfo {
            path: device_path.to_path_buf(),
            name,
            size,
            size_human: format_bytes(size),
            model: device_fields.get("MODEL").map(|s| s.trim().to_string()),
            vendor: device_fields.get("VENDOR").map(|s| s.trim().to_string()),
            removable,
            partitions,
        })
    }
}

fn parse_lsblk_line(line: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let re = Regex::new(r#"(\w+)="([^"]*)""#).unwrap();

    for cap in re.captures_iter(line) {
        let key = cap[1].to_string();
        let value = cap[2].to_string();
        map.insert(key, value);
    }

    map
}

fn parse_partition(fields: &std::collections::HashMap<String, String>) -> Option<PartitionInfo> {
    let name = fields.get("NAME")?;
    let size = fields.get("SIZE")?.parse::<u64>().ok()?;

    // Extract partition number from name (e.g., sdb1 -> 1)
    let number = name.chars()
        .rev()
        .take_while(|c| c.is_numeric())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>()
        .parse::<u32>()
        .ok()?;

    let path = PathBuf::from("/dev").join(name);
    let filesystem = fields.get("FSTYPE").filter(|s| !s.is_empty()).cloned();
    let label = fields.get("LABEL").filter(|s| !s.is_empty()).cloned();
    let mounted = fields.get("MOUNTPOINT")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    Some(PartitionInfo {
        path,
        number,
        size,
        filesystem,
        label,
        mounted,
    })
}

/// Check if a device or any of its partitions are mounted
pub fn is_device_mounted<P: AsRef<Path>>(device: P) -> Result<bool> {
    let info = DeviceInfo::from_device(device)?;
    Ok(info.partitions.iter().any(|p| p.mounted.is_some()))
}

/// Unmount all partitions of a device
pub fn unmount_device<P: AsRef<Path>>(device: P) -> Result<()> {
    let info = DeviceInfo::from_device(device)?;

    for partition in &info.partitions {
        if partition.mounted.is_some() {
            let output = Command::new("umount")
                .arg(&partition.path)
                .output()
                .map_err(|e| MultiBootError::CommandFailed {
                    cmd: "umount".to_string(),
                    message: e.to_string(),
                })?;

            if !output.status.success() {
                return Err(MultiBootError::DeviceMounted(
                    partition.path.display().to_string()
                ));
            }
        }
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}
