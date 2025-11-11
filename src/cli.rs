use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "multi-boot")]
#[command(version, about, long_about = None)]
#[command(author = "Multi-Boot Contributors")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Skip confirmation prompts
    #[arg(short, long, global = true)]
    pub yes: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List available USB devices
    Devices {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Initialize a USB drive for multi-boot
    Init {
        /// Device path (e.g., /dev/sdb)
        device: String,

        /// Volume label for the data partition
        #[arg(short, long, default_value = "MULTIBOOT")]
        label: String,

        /// Skip BIOS boot support (UEFI only)
        #[arg(long)]
        uefi_only: bool,
    },

    /// Add an ISO to the USB drive
    Add {
        /// Device path or mount point
        device: String,

        /// Path to ISO file
        iso: PathBuf,

        /// Custom name for the boot entry
        #[arg(short, long)]
        name: Option<String>,
    },

    /// List ISOs on the USB drive
    List {
        /// Device path or mount point
        device: String,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Remove an ISO from the USB drive
    Remove {
        /// Device path or mount point
        device: String,

        /// ISO filename to remove
        iso: String,
    },

    /// Regenerate the boot configuration
    Refresh {
        /// Device path or mount point
        device: String,
    },

    /// Check system requirements
    Check {
        /// Fix missing dependencies if possible
        #[arg(short, long)]
        fix: bool,
    },
}
