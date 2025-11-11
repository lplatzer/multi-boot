mod grub;
mod config;

pub use grub::{install_grub, check_grub_tools};
pub use config::{generate_grub_config, GrubConfig, IsoEntry};

use crate::Result;

/// Check if all required bootloader tools are available
pub fn check_bootloader_requirements() -> Result<Vec<String>> {
    check_grub_tools()
}
