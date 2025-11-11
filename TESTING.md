# Testing Guide for Multi-Boot USB Tool

This guide will help you test the multi-boot USB tool safely and effectively.

## ⚠️ Safety First

**NEVER test on your main system drive!** Always use:
- A dedicated USB drive (all data will be erased)
- A spare/test computer for boot testing
- Virtual machines for initial testing (with USB passthrough)

## Prerequisites

### Required Hardware
- USB drive (8GB minimum, 16GB+ recommended)
- Test computer with:
  - UEFI support (for UEFI testing)
  - Legacy BIOS support (for BIOS testing)
  - Ability to boot from USB
- **OR** Virtual machine software (VirtualBox, QEMU, VMware)

### Required Software
All dependencies from README.md must be installed:
```bash
sudo multi-boot check
```

### Test ISO Files
Download test ISOs (smaller ones for faster testing):
- Ubuntu 24.04 Desktop (or minimal)
- Fedora Workstation
- Arch Linux
- Windows 11 (optional, large file)

## Testing Workflow

### Phase 1: Environment Setup

#### 1.1 Identify Test USB Drive

```bash
# List all devices BEFORE plugging in USB
lsblk

# Plug in USB drive

# List devices AFTER plugging in USB
lsblk

# Note the device path (e.g., /dev/sdb)
# Make ABSOLUTELY SURE this is your USB drive!
```

#### 1.2 Verify Device Selection

```bash
# Check device details
sudo fdisk -l /dev/sdX

# Verify it's removable
cat /sys/block/sdX/removable  # Should output "1"

# Check size matches your USB
lsblk -b /dev/sdX
```

### Phase 2: Basic Functionality Tests

#### 2.1 System Check

```bash
# Test: Verify all requirements are met
sudo multi-boot check

# Expected: All items should show [✓]
# If any show [ ], install the missing package
```

#### 2.2 Device Listing

```bash
# Test: List devices
multi-boot devices

# Expected: USB drive should appear in list
# Verify: "Removable: Yes"

# Test: Detailed listing
multi-boot devices --detailed

# Expected: Should show partitions (if any exist)
```

#### 2.3 USB Initialization (UEFI + BIOS)

```bash
# Test: Initialize USB with hybrid boot support
sudo multi-boot init /dev/sdX

# Expected:
# - Confirmation prompt appears
# - Progress messages for each step
# - Success message at the end

# Verify partitions were created
lsblk /dev/sdX

# Expected output:
# sdX
# ├─sdX1  (512M, vfat, label: ESP)
# ├─sdX2  (1M, no filesystem)
# └─sdX3  (rest, exfat, label: MULTIBOOT)

# Verify ESP partition structure
sudo mount /dev/sdX1 /mnt
ls -R /mnt
# Expected: /mnt/EFI/BOOT/BOOTX64.EFI and /mnt/boot/grub/grub.cfg should exist
sudo umount /mnt
```

#### 2.4 USB Initialization (UEFI Only)

```bash
# Test: Initialize USB with UEFI-only support
sudo multi-boot init /dev/sdX --uefi-only

# Verify only 2 partitions
lsblk /dev/sdX

# Expected:
# sdX
# ├─sdX1  (512M, vfat, label: ESP)
# └─sdX2  (rest, exfat, label: MULTIBOOT)
```

### Phase 3: ISO Management Tests

#### 3.1 Add ISOs

```bash
# Test: Add first ISO
sudo multi-boot add /dev/sdX ~/Downloads/ubuntu-24.04-desktop-amd64.iso

# Expected:
# - ISO validation message
# - Progress bar during copy
# - Boot configuration update
# - Success message

# Test: Add second ISO
sudo multi-boot add /dev/sdX ~/Downloads/fedora-40-workstation.iso

# Test: Add with custom name
sudo multi-boot add /dev/sdX ~/Downloads/arch-2024.11.iso --name "Arch Linux"
```

#### 3.2 List ISOs

```bash
# Test: Simple list
multi-boot list /dev/sdX

# Expected: All added ISOs should be listed

# Test: Detailed list
multi-boot list /dev/sdX --detailed

# Expected: ISOs with sizes should be listed
```

#### 3.3 Verify ISO Files

```bash
# Mount data partition and check files
sudo mount /dev/sdX3 /mnt  # or sdX2 for UEFI-only
ls -lh /mnt/isos/

# Expected: All ISO files should be present with correct sizes

# Verify GRUB config was generated
sudo mount /dev/sdX1 /mnt/esp
cat /mnt/esp/boot/grub/grub.cfg

# Expected: Boot entries for each ISO
# Check for:
# - menuentry for each ISO
# - loopback commands
# - appropriate kernel/initrd paths

sudo umount /mnt/esp /mnt
```

#### 3.4 Remove ISOs

```bash
# Test: Remove an ISO
sudo multi-boot remove /dev/sdX ubuntu-24.04-desktop-amd64.iso

# Expected:
# - Confirmation prompt
# - Removal message
# - Boot configuration update

# Verify removal
multi-boot list /dev/sdX

# Expected: ISO should not be in list

# Verify file was deleted
sudo mount /dev/sdX3 /mnt
ls /mnt/isos/
sudo umount /mnt
```

#### 3.5 Refresh Configuration

```bash
# Test: Refresh boot configuration
sudo multi-boot refresh /dev/sdX

# Expected: Success message

# Verify: Boot configuration should be regenerated
```

### Phase 4: Error Handling Tests

#### 4.1 Permission Tests

```bash
# Test: Run without sudo (should fail gracefully)
multi-boot init /dev/sdX

# Expected: Clear error message about needing root privileges
```

#### 4.2 Invalid Device Tests

```bash
# Test: Non-existent device
sudo multi-boot init /dev/sdZZZ

# Expected: Error message "Device not found"

# Test: Non-removable device (be VERY careful here!)
# DO NOT actually run this on a system drive!
# Expected: Error "Device is not removable"
```

#### 4.3 Invalid ISO Tests

```bash
# Test: Non-existent ISO
sudo multi-boot add /dev/sdX /path/to/nonexistent.iso

# Expected: Error "ISO file not found"

# Test: Invalid ISO file (create a dummy file)
echo "not an iso" > /tmp/fake.iso
sudo multi-boot add /dev/sdX /tmp/fake.iso

# Expected: Error "Invalid ISO file"
```

#### 4.4 Insufficient Space Test

```bash
# Test: Try to add an ISO larger than available space
# (Use a very small USB drive or add many large ISOs)

# Expected: Clear error message about insufficient space
```

#### 4.5 Duplicate ISO Test

```bash
# Add the same ISO twice
sudo multi-boot add /dev/sdX ~/Downloads/ubuntu-24.04.iso
sudo multi-boot add /dev/sdX ~/Downloads/ubuntu-24.04.iso

# Expected: Error "ISO already exists"
```

### Phase 5: Boot Tests

#### 5.1 Virtual Machine Boot Test (Recommended First)

Using VirtualBox:

```bash
# Create a VM with UEFI support
VBoxManage createvm --name "MultiBootTest" --ostype Ubuntu_64 --register
VBoxManage modifyvm "MultiBootTest" --firmware efi --memory 2048
VBoxManage modifyvm "MultiBootTest" --boot1 disk

# Attach USB device
# (Use VirtualBox GUI to attach the USB device)

# Start VM and test boot
VBoxManage startvm "MultiBootTest"
```

Expected:
1. GRUB menu should appear
2. All added ISOs should be listed
3. Reboot/Shutdown options should work
4. Selecting an ISO should start the boot process

#### 5.2 UEFI Boot Test (Physical Hardware)

1. Insert USB drive into test computer
2. Enter BIOS/UEFI settings (usually F2, F12, or DEL during boot)
3. Ensure UEFI boot is enabled
4. Set USB drive as first boot device
5. Save and reboot

Expected:
- GRUB menu appears with graphical terminal
- All ISOs listed
- Can navigate with arrow keys
- Selecting an ISO boots it successfully

**Test Matrix:**

| ISO Type | Expected Behavior |
|----------|------------------|
| Ubuntu   | Should boot to live desktop |
| Fedora   | Should boot to live desktop |
| Arch     | Should boot to installation |
| Windows  | Should start Windows installer |

#### 5.3 BIOS/Legacy Boot Test

1. Insert USB drive into test computer
2. Enter BIOS settings
3. Ensure Legacy/CSM boot is enabled
4. Disable UEFI boot
5. Set USB drive as first boot device
6. Save and reboot

Expected:
- Same behavior as UEFI boot
- GRUB menu appears (text mode)
- ISOs boot successfully

### Phase 6: Edge Cases and Stress Tests

#### 6.1 Many ISOs Test

```bash
# Add 10+ ISOs (if you have space)
for iso in ~/Downloads/*.iso; do
    sudo multi-boot add /dev/sdX "$iso"
done

# Verify all appear in boot menu
multi-boot list /dev/sdX --detailed

# Boot and check menu scrolling works
```

#### 6.2 Large ISO Test

```bash
# Add a very large ISO (Windows 11 is ~5GB)
sudo multi-boot add /dev/sdX ~/Downloads/windows-11.iso

# Verify progress bar works
# Verify copy completes successfully
```

#### 6.3 Special Characters Test

```bash
# Test with ISO that has spaces/special chars in name
cp ~/Downloads/ubuntu.iso ~/Downloads/"Ubuntu 24.04 (Desktop).iso"
sudo multi-boot add /dev/sdX ~/Downloads/"Ubuntu 24.04 (Desktop).iso"

# Verify it's added correctly
multi-boot list /dev/sdX
```

#### 6.4 Concurrent Operations Test

```bash
# Try to add two ISOs at once (in different terminals)
# Terminal 1:
sudo multi-boot add /dev/sdX ~/Downloads/iso1.iso

# Terminal 2 (while first is running):
sudo multi-boot add /dev/sdX ~/Downloads/iso2.iso

# Expected: Second one should wait or fail gracefully
```

#### 6.5 Interrupted Operation Test

```bash
# Start adding a large ISO
sudo multi-boot add /dev/sdX ~/Downloads/large.iso

# Press Ctrl+C during copy

# Verify device is still in consistent state
lsblk /dev/sdX
multi-boot list /dev/sdX

# Re-run init if needed
sudo multi-boot init /dev/sdX
```

### Phase 7: Compatibility Tests

#### 7.1 Cross-Platform Boot

Test the USB drive on different computers:
- [ ] Modern UEFI laptop
- [ ] Old BIOS desktop
- [ ] Mac (if available, UEFI mode)
- [ ] ARM device (if ARM support is added)

#### 7.2 Different BIOS Vendors

Test on computers with:
- [ ] AMI BIOS
- [ ] Award BIOS
- [ ] Phoenix BIOS
- [ ] UEFI implementations (Dell, HP, Lenovo, etc.)

## Test Checklist

Use this checklist to track your testing progress:

### Basic Functionality
- [ ] `multi-boot check` shows all requirements met
- [ ] `multi-boot devices` lists USB drive correctly
- [ ] `multi-boot init` creates partitions correctly
- [ ] `multi-boot init --uefi-only` works
- [ ] `multi-boot add` copies ISO successfully
- [ ] `multi-boot list` shows all ISOs
- [ ] `multi-boot remove` deletes ISO correctly
- [ ] `multi-boot refresh` regenerates config

### Error Handling
- [ ] Proper error for missing root privileges
- [ ] Proper error for invalid device
- [ ] Proper error for invalid ISO
- [ ] Proper error for insufficient space
- [ ] Proper error for duplicate ISO
- [ ] Graceful handling of Ctrl+C

### Boot Tests
- [ ] UEFI boot works
- [ ] BIOS/Legacy boot works
- [ ] GRUB menu displays correctly
- [ ] Ubuntu ISO boots
- [ ] Fedora ISO boots
- [ ] Arch ISO boots
- [ ] Windows ISO boots (if tested)
- [ ] Menu navigation works
- [ ] Reboot option works
- [ ] Shutdown option works

### Advanced Tests
- [ ] Multiple ISOs work
- [ ] Large ISOs work
- [ ] Special characters in filenames work
- [ ] Boots on different computers
- [ ] Boots on different BIOS/UEFI vendors

## Reporting Issues

If you find bugs during testing, report them with:

1. **System information:**
   ```bash
   uname -a
   lsb_release -a  # or cat /etc/os-release
   ```

2. **USB drive information:**
   ```bash
   lsblk /dev/sdX
   sudo fdisk -l /dev/sdX
   ```

3. **Command run and output:**
   ```bash
   # Include the full command and output
   ```

4. **Expected vs actual behavior**

5. **Steps to reproduce**

## Automated Testing (Future)

For developers contributing to the project:

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin
```

## Performance Benchmarks

Track performance metrics:

```bash
# Time to initialize USB
time sudo multi-boot init /dev/sdX -y

# Time to add 1GB ISO
time sudo multi-boot add /dev/sdX small.iso

# Time to add 5GB ISO
time sudo multi-boot add /dev/sdX large.iso
```

Expected times (will vary by USB speed):
- Init: 30-60 seconds
- Add 1GB ISO: 1-3 minutes
- Add 5GB ISO: 5-15 minutes

## Clean Up After Testing

```bash
# Unmount any mounted partitions
sudo umount /dev/sdX*

# Optional: Wipe USB drive completely
sudo dd if=/dev/zero of=/dev/sdX bs=1M count=100

# Or just reinitialize with a standard filesystem
sudo mkfs.vfat /dev/sdX1
```

## Getting Help

If you encounter issues during testing:
1. Check the README.md troubleshooting section
2. Review ARCHITECTURE.md for technical details
3. Open an issue on GitHub with test details
4. Join discussions for community help
