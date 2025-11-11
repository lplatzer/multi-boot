# Multi-ISO Boot Tool - Architecture

## Overview
A Rust-based tool to create bootable USB drives that can boot multiple ISO images on both BIOS/Legacy and UEFI systems.

## Design Goals
- **Multi-boot**: Support multiple ISO images on a single USB drive
- **Dual boot mode**: BIOS/Legacy and UEFI compatibility
- **Cross-platform ISOs**: Windows, Linux (various distros), and other bootable images
- **Large file support**: exFAT for ISOs >4GB
- **User-friendly**: Simple CLI interface
- **Safe**: Verify operations, prevent accidental data loss

## Architecture

### 1. Disk Layout
```
GPT Partition Table (with protective MBR for BIOS compatibility)
├── Partition 1: EFI System Partition (ESP)
│   ├── Type: EF00 (EFI System)
│   ├── Size: 512MB
│   ├── Filesystem: FAT32
│   └── Mount: /boot/efi
│       ├── EFI/
│       │   └── BOOT/
│       │       ├── BOOTX64.EFI (UEFI bootloader)
│       │       └── grubx64.efi
│       └── grub/
│           ├── grub.cfg (boot menu config)
│           └── grubenv
│
├── Partition 2: BIOS Boot Partition
│   ├── Type: EF02 (BIOS boot)
│   ├── Size: 1MB
│   └── Contains: GRUB core image for BIOS
│
└── Partition 3: ISO Storage
    ├── Type: 0700 (Microsoft basic data)
    ├── Size: Remaining space
    ├── Filesystem: exFAT (supports files >4GB)
    └── Mount: /isos
        ├── isos/
        │   ├── ubuntu-24.04-desktop-amd64.iso
        │   ├── windows-11.iso
        │   └── fedora-workstation.iso
        └── boot/
            └── grub/
                └── grub.cfg (generated from ISO scan)
```

### 2. Boot Flow

#### BIOS Boot:
1. BIOS reads protective MBR
2. MBR loads GRUB core from BIOS boot partition (Part 2)
3. GRUB loads `grub.cfg` from data partition
4. User selects ISO from menu
5. GRUB uses loopback to mount and boot selected ISO

#### UEFI Boot:
1. UEFI firmware reads ESP (Part 1)
2. Loads `/EFI/BOOT/BOOTX64.EFI` (GRUB EFI)
3. GRUB loads `grub.cfg` from ESP or data partition
4. User selects ISO from menu
5. GRUB uses loopback to mount and boot selected ISO

### 3. Module Structure

```
multi-boot/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── cli.rs               # Command-line interface (clap)
│   ├── device/
│   │   ├── mod.rs           # Device management module
│   │   ├── detect.rs        # USB device detection
│   │   └── info.rs          # Device information
│   ├── disk/
│   │   ├── mod.rs           # Disk operations module
│   │   ├── partition.rs     # GPT/MBR partitioning
│   │   └── filesystem.rs    # Format filesystems
│   ├── bootloader/
│   │   ├── mod.rs           # Bootloader module
│   │   ├── grub.rs          # GRUB installation
│   │   └── config.rs        # GRUB config generation
│   ├── iso/
│   │   ├── mod.rs           # ISO management module
│   │   ├── manager.rs       # Add/remove/list ISOs
│   │   └── detector.rs      # ISO metadata detection
│   ├── config/
│   │   ├── mod.rs           # Configuration module
│   │   └── settings.rs      # Tool settings
│   └── error.rs             # Error types
├── Cargo.toml
├── README.md
└── ARCHITECTURE.md
```

### 4. Key Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
anyhow = "1"
thiserror = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = "1"
walkdir = "2"
indicatif = "0.17"  # Progress bars
```

### 5. External Tools Required
- `parted` or `gdisk`: Partition management
- `mkfs.vfat`: FAT32 formatting
- `mkfs.exfat`: exFAT formatting
- `grub-install`: GRUB bootloader installation
- `grub-mkconfig`: GRUB configuration

### 6. GRUB Configuration Strategy

The tool generates a dynamic `grub.cfg` that:
1. Scans `/isos` directory for ISO files
2. Detects ISO type (Linux, Windows, etc.) via heuristics
3. Generates appropriate boot entries using loopback:

```grub
# Example generated entry
menuentry "Ubuntu 24.04 Desktop" {
    set isofile="/isos/ubuntu-24.04-desktop-amd64.iso"
    loopback loop $isofile
    linux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile quiet splash
    initrd (loop)/casper/initrd
}
```

### 7. ISO Detection Heuristics

**Linux Detection:**
- Look for `/casper/`, `/live/`, `/isolinux/` directories
- Parse `.disk/info` or `README` files
- Extract kernel and initrd paths

**Windows Detection:**
- Look for `/sources/boot.wim`
- Check for `/bootmgr` or `/efi/microsoft/`
- Use chainloading via GRUB

**Generic Fallback:**
- Use ISO boot sector chainload
- Attempt MEMDISK (for older ISOs)

### 8. Commands

```
multi-boot init <device>              # Initialize USB drive
multi-boot add <device> <iso-path>    # Add ISO to drive
multi-boot list <device>              # List ISOs on drive
multi-boot remove <device> <iso>      # Remove ISO from drive
multi-boot refresh <device>           # Regenerate boot config
multi-boot devices                    # List available devices
```

### 9. Safety Features
- Confirm before destructive operations
- Verify device is removable (USB)
- Check for mounted filesystems
- Backup partition table before modifications
- Validate ISO integrity (checksum if provided)

### 10. Future Enhancements (Post-MVP)
- Persistence support for Linux live USBs
- Automatic ISO download from URLs
- GUI interface (using egui or similar)
- Secure Boot support
- Custom boot menu themes
- Multi-architecture support (ARM)

## Implementation Phases

### Phase 1: Core Infrastructure (Current MVP)
- Device detection and management
- Partitioning and filesystem creation
- GRUB installation (BIOS + UEFI)
- Basic ISO management
- CLI interface

### Phase 2: Enhanced Features
- Automatic ISO detection and configuration
- Better error handling and recovery
- Progress indicators
- Comprehensive testing

### Phase 3: Polish
- Documentation
- Testing guide
- Example workflows
- Performance optimization
