# Multi-Boot USB Tool

A Rust-based tool to create bootable USB drives that can boot multiple ISO images on both BIOS/Legacy and UEFI systems. Available in both CLI and GUI versions.

## Features

- **Multi-ISO Boot**: Store and boot multiple ISO images from a single USB drive
- **Dual Boot Mode**: Supports both BIOS/Legacy and UEFI boot modes
- **Cross-Platform ISOs**: Boot Windows, Linux (Ubuntu, Debian, Fedora, Arch), and other distributions
- **Large File Support**: Uses exFAT for data partition to support ISOs >4GB
- **Auto-Configuration**: Automatically detects ISO types and generates appropriate boot entries
- **Safe Operations**: Verifies devices are removable and confirms destructive operations
- **Dual Interface**:
  - **CLI**: Powerful command-line interface for automation and scripting
  - **GUI**: User-friendly graphical interface for desktop users

## Quick Start

### GUI (Recommended for Desktop Users)

```bash
# Run the GUI application
sudo cargo run --features gui --bin multi-boot-gui
```

The GUI provides a point-and-click interface for:
- Selecting USB devices
- Initializing devices
- Adding and removing ISOs
- Real-time progress indicators

### CLI (For Automation and Scripting)

```bash
# Check system requirements
sudo cargo run -- check

# List available USB devices
cargo run -- devices

# Initialize a USB drive (WARNING: This erases all data!)
sudo cargo run -- init /dev/sdX

# Add an ISO to the drive
sudo cargo run -- add /dev/sdX /path/to/ubuntu-24.04-desktop-amd64.iso

# List ISOs on the drive
cargo run -- list /dev/sdX

# Remove an ISO
sudo cargo run -- remove /dev/sdX ubuntu-24.04-desktop-amd64.iso

# Regenerate boot configuration
sudo cargo run -- refresh /dev/sdX
```

## Installation

### Prerequisites

#### Debian/Ubuntu
```bash
sudo apt install build-essential parted dosfstools exfatprogs \
                 grub2 grub-pc-bin grub-efi-amd64-bin util-linux
```

#### Fedora/RHEL
```bash
sudo dnf install gcc parted dosfstools exfatprogs \
                 grub2 grub2-pc grub2-efi-x64 util-linux
```

#### Arch Linux
```bash
sudo pacman -S base-devel parted dosfstools exfatprogs \
               grub util-linux
```

### Build from Source

```bash
git clone https://github.com/lplatzer/multi-boot.git
cd multi-boot

# Build CLI only (smaller, faster)
cargo build --release
sudo cp target/release/multi-boot /usr/local/bin/

# Build with GUI (recommended for desktop)
cargo build --release --features gui
sudo cp target/release/multi-boot /usr/local/bin/
sudo cp target/release/multi-boot-gui /usr/local/bin/
```

## Usage

### 1. Check System Requirements

Before starting, verify that all required tools are installed:

```bash
sudo multi-boot check
```

This will check for:
- Root privileges
- Partitioning tools (parted)
- Filesystem tools (mkfs.vfat, mkfs.exfat)
- GRUB bootloader (BIOS and UEFI modules)
- System utilities (wipefs, lsblk, mount, umount)

### 2. List USB Devices

Find your USB device:

```bash
multi-boot devices
```

For detailed information including partitions:

```bash
multi-boot devices --detailed
```

### 3. Initialize USB Drive

**⚠️ WARNING: This will ERASE ALL DATA on the device!**

Initialize a USB drive with multi-boot support:

```bash
# UEFI + BIOS support (recommended)
sudo multi-boot init /dev/sdX

# UEFI only
sudo multi-boot init /dev/sdX --uefi-only

# Custom volume label
sudo multi-boot init /dev/sdX --label "MY_MULTIBOOT"

# Skip confirmation prompt
sudo multi-boot init /dev/sdX -y
```

This will:
1. Create a GPT partition table with protective MBR
2. Create three partitions:
   - EFI System Partition (512MB, FAT32)
   - BIOS Boot Partition (1MB, only in hybrid mode)
   - Data Partition (remaining space, exFAT)
3. Install GRUB for both BIOS and UEFI
4. Generate initial boot configuration

### 4. Add ISOs

Add ISO files to the USB drive:

```bash
# Add an ISO
sudo multi-boot add /dev/sdX /path/to/ubuntu-24.04.iso

# Add with custom name
sudo multi-boot add /dev/sdX /path/to/fedora-40.iso --name "Fedora 40"
```

The tool will:
- Validate the ISO file
- Check available space
- Copy the ISO with a progress bar
- Regenerate the boot configuration

### 5. List ISOs

View all ISOs on the USB drive:

```bash
# Simple list
multi-boot list /dev/sdX

# Detailed information (sizes, etc.)
multi-boot list /dev/sdX --detailed
```

### 6. Remove ISOs

Remove an ISO from the USB drive:

```bash
sudo multi-boot remove /dev/sdX ubuntu-24.04.iso
```

### 7. Refresh Boot Configuration

If you manually added/removed ISOs or want to regenerate the boot menu:

```bash
sudo multi-boot refresh /dev/sdX
```

## Supported Distributions

The tool automatically detects and configures boot entries for:

- **Ubuntu** (and variants: Kubuntu, Xubuntu, Lubuntu)
- **Debian** (and Linux Mint)
- **Fedora**
- **Arch Linux** (and Manjaro)
- **Windows** (Windows 10, Windows 11)
- **Generic Linux** distributions

For unsupported distributions, the tool will create a fallback boot entry.

## How It Works

### Disk Layout

```
GPT Partition Table (with protective MBR)
├── Partition 1: EFI System Partition (ESP)
│   ├── Size: 512MB
│   ├── Filesystem: FAT32
│   └── Contents: GRUB EFI bootloader, boot configuration
│
├── Partition 2: BIOS Boot Partition (hybrid mode only)
│   ├── Size: 1MB
│   └── Contents: GRUB core image for BIOS
│
└── Partition 3: Data Partition
    ├── Size: Remaining space
    ├── Filesystem: exFAT
    └── Contents: ISO files in /isos directory
```

### Boot Process

#### UEFI Boot:
1. UEFI firmware loads `/EFI/BOOT/BOOTX64.EFI` from ESP
2. GRUB loads `grub.cfg` and displays boot menu
3. User selects an ISO
4. GRUB uses loopback to mount and boot the selected ISO

#### BIOS Boot:
1. BIOS loads GRUB from the MBR/BIOS boot partition
2. GRUB loads `grub.cfg` from ESP
3. Same as UEFI from this point

### ISO Detection

The tool uses filename heuristics to detect ISO types:
- Looks for keywords like "ubuntu", "fedora", "windows", etc.
- Generates appropriate GRUB boot entries based on the detected type
- For Linux ISOs, uses loopback mounting with distribution-specific parameters
- For Windows ISOs, uses chainloading

## Troubleshooting

### USB Drive Not Detected

Make sure the USB drive is:
- Plugged in
- Not mounted (the tool will try to unmount it)
- Recognized by the system (`lsblk` should show it)

### Permission Denied

Most operations require root privileges. Always run with `sudo`:

```bash
sudo multi-boot <command>
```

### ISO Won't Boot

1. **Check ISO compatibility**: Not all ISOs support loopback booting
2. **Try refreshing**: `sudo multi-boot refresh /dev/sdX`
3. **Check BIOS/UEFI settings**: Ensure Secure Boot is disabled
4. **Test the ISO**: Try the ISO individually with other tools (like Rufus or Ventoy)

### GRUB Error

If you see GRUB errors during boot:
1. The device might not have been properly initialized
2. Try reinitializing: `sudo multi-boot init /dev/sdX`
3. Check that GRUB modules are installed

### Insufficient Space

The tool checks available space before copying. If you get this error:
- Remove some ISOs: `sudo multi-boot remove /dev/sdX <iso-name>`
- Or use a larger USB drive

### Device Appears to be a System Disk

The tool refuses to operate on non-removable devices for safety. If you're sure you want to use a device:
1. Double-check you have the correct device path
2. The device must be marked as removable in the system

## Advanced Usage

### Custom Boot Entries

The boot configuration is stored in `/boot/grub/grub.cfg` on the ESP partition. You can manually edit it if needed:

```bash
# Mount the ESP partition
sudo mount /dev/sdX1 /mnt

# Edit grub.cfg
sudo nano /mnt/boot/grub/grub.cfg

# Unmount
sudo umount /mnt
```

After manual edits, test by rebooting. If something breaks, run:

```bash
sudo multi-boot refresh /dev/sdX
```

### Using Different File Systems

The tool uses:
- **FAT32** for ESP (required for UEFI)
- **exFAT** for data (supports large files, widely compatible)

If you need NTFS support, you can manually reformat the data partition, but you'll need to modify the GRUB configuration to support it.

## Comparison with Ventoy

**Similarities:**
- Both support multi-ISO booting
- Both support BIOS and UEFI
- Both use GRUB as the bootloader

**Differences:**
- **Multi-Boot** is written in Rust, Ventoy is written in C
- **Multi-Boot** gives you more control over the disk layout
- **Multi-Boot** is simpler and more transparent
- **Ventoy** has more features (plugins, persistence, themes)
- **Ventoy** has better ISO compatibility out of the box

**When to use Multi-Boot:**
- You want a simple, transparent solution
- You want to understand and control the boot process
- You're comfortable with command-line tools
- You want a Rust-based project you can hack on

**When to use Ventoy:**
- You want maximum ISO compatibility
- You need advanced features like persistence
- You prefer a GUI
- You don't want to tinker

## Development

### Project Structure

```
multi-boot/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── cli.rs               # Command-line interface
│   ├── error.rs             # Error types
│   ├── device/              # Device detection and management
│   ├── disk/                # Partitioning and filesystems
│   ├── bootloader/          # GRUB installation and configuration
│   ├── iso/                 # ISO management
│   └── config/              # Configuration
├── Cargo.toml
├── README.md
└── ARCHITECTURE.md          # Detailed architecture documentation
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- <command>
```

### Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Safety and Warnings

**⚠️ IMPORTANT WARNINGS:**

1. **Data Loss**: Initializing a device (`init` command) will ERASE ALL DATA on that device
2. **Device Selection**: Always double-check you're using the correct device path
3. **System Disk**: Never run this tool on your system disk (the tool will try to prevent this)
4. **Backups**: Always backup important data before using this tool
5. **Root Access**: This tool requires root access to modify disks

**The authors are not responsible for any data loss. Use at your own risk.**

## License

This project is dual-licensed under:
- MIT License
- Apache License 2.0

Choose the license that best suits your needs.

## Acknowledgments

- GRUB bootloader team
- The Rust community
- Ventoy project for inspiration
- All contributors

## Support

- **Issues**: https://github.com/lplatzer/multi-boot/issues
- **Discussions**: https://github.com/lplatzer/multi-boot/discussions

## Roadmap

Planned features for future releases:

- [ ] Persistence support for Linux live USBs
- [ ] Automatic ISO download from URLs
- [ ] GUI interface (using egui)
- [ ] Secure Boot support
- [ ] Custom boot menu themes
- [ ] Better ISO detection (parse ISO filesystem)
- [ ] Multi-architecture support (ARM)
- [ ] Windows GUI wrapper
- [ ] ISO integrity verification
- [ ] Incremental updates

## FAQ

**Q: Can I use this on Windows/macOS?**
A: Currently, the tool is designed for Linux. Windows and macOS support may be added in the future.

**Q: Will this work with Secure Boot?**
A: Not yet. Secure Boot support is planned for a future release.

**Q: Can I boot from multiple USBs?**
A: Yes! Just initialize each USB separately.

**Q: How do I reset a USB to normal?**
A: Use a partitioning tool (like GParted) to delete all partitions and create a new one, or re-initialize with this tool.

**Q: Why Rust?**
A: Rust provides memory safety, great error handling, and excellent tooling, making it ideal for system tools.

**Q: Can I contribute?**
A: Absolutely! Contributions are welcome. See the Contributing section above.
