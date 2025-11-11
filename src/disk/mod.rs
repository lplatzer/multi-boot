mod partition;
mod filesystem;

pub use partition::{create_partitions, PartitionScheme};
pub use filesystem::{format_partitions, mount_partition, unmount_partition, check_filesystem_tools};

use crate::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiskLayout {
    pub device: PathBuf,
    pub esp_partition: PathBuf,      // EFI System Partition
    pub bios_partition: PathBuf,     // BIOS boot partition
    pub data_partition: PathBuf,     // Data partition for ISOs
    pub esp_mountpoint: PathBuf,
    pub data_mountpoint: PathBuf,
}

impl DiskLayout {
    pub fn from_device<P: AsRef<Path>>(device: P, uefi_only: bool) -> Self {
        let device_path = device.as_ref();
        let device_name = device_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sdb");

        // Determine partition naming scheme (e.g., /dev/sdb1 vs /dev/nvme0n1p1)
        let (p1, p2, p3) = if device_name.starts_with("nvme") ||
                              device_name.starts_with("mmcblk") {
            (
                device_path.join("p1"),
                device_path.join("p2"),
                device_path.join("p3"),
            )
        } else {
            (
                PathBuf::from(format!("{}1", device_path.display())),
                PathBuf::from(format!("{}2", device_path.display())),
                PathBuf::from(format!("{}3", device_path.display())),
            )
        };

        Self {
            device: device_path.to_path_buf(),
            esp_partition: p1,
            bios_partition: if uefi_only { PathBuf::new() } else { p2.clone() },
            data_partition: if uefi_only { p2 } else { p3 },
            esp_mountpoint: PathBuf::from("/tmp/multiboot-esp"),
            data_mountpoint: PathBuf::from("/tmp/multiboot-data"),
        }
    }

    pub fn create_mountpoints(&self) -> Result<()> {
        std::fs::create_dir_all(&self.esp_mountpoint)?;
        std::fs::create_dir_all(&self.data_mountpoint)?;
        Ok(())
    }

    pub fn cleanup_mountpoints(&self) -> Result<()> {
        let _ = std::fs::remove_dir(&self.esp_mountpoint);
        let _ = std::fs::remove_dir(&self.data_mountpoint);
        Ok(())
    }
}
