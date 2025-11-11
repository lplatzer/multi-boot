mod detect;
mod info;

pub use detect::{list_devices, find_device, verify_device};
pub use info::{DeviceInfo, is_device_mounted, unmount_device};

use crate::Result;
use std::path::Path;

/// Check if running with root privileges
pub fn check_root() -> Result<()> {
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        return Err(crate::MultiBootError::PermissionDenied);
    }
    Ok(())
}

/// Validate that a device path exists and is a block device
pub fn validate_device<P: AsRef<Path>>(device: P) -> Result<()> {
    let path = device.as_ref();

    if !path.exists() {
        return Err(crate::MultiBootError::DeviceNotFound(
            path.display().to_string()
        ));
    }

    // Check if it's a block device
    let metadata = std::fs::metadata(path)?;
    let file_type = metadata.file_type();

    // On Linux, we can check if it's a block device
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::FileTypeExt;
        if !file_type.is_block_device() {
            return Err(crate::MultiBootError::Other(
                format!("{} is not a block device", path.display())
            ));
        }
    }

    Ok(())
}
