# Contributing to Multi-Boot USB Tool

Thank you for your interest in contributing to the Multi-Boot USB Tool! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Coding Standards](#coding-standards)
- [Documentation](#documentation)

## Code of Conduct

Be respectful, inclusive, and constructive. We want this to be a welcoming community for everyone.

## Getting Started

### Areas We Need Help With

- **Core functionality improvements**
  - Better ISO detection algorithms
  - Support for more distributions
  - Improved boot configuration generation

- **New features**
  - Persistence support for Linux live USBs
  - GUI interface
  - Secure Boot support
  - Windows/macOS support

- **Testing**
  - Test on different hardware
  - Test with different distributions
  - Write automated tests

- **Documentation**
  - Improve README and guides
  - Add translations
  - Create video tutorials

- **Bug fixes**
  - Check the issue tracker for reported bugs
  - Test and reproduce issues
  - Submit fixes

## Development Setup

### Prerequisites

1. **Rust toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **System dependencies** (see README.md)
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential parted dosfstools exfatprogs \
                    grub2 grub-pc-bin grub-efi-amd64-bin util-linux
   ```

3. **USB drive for testing** (8GB minimum, will be erased!)

### Clone and Build

```bash
# Fork the repository on GitHub first, then:
git clone https://github.com/YOUR_USERNAME/multi-boot.git
cd multi-boot

# Build the project
cargo build

# Run tests
cargo test

# Try it out (be careful!)
sudo cargo run -- check
```

## Making Changes

### Before You Start

1. **Check existing issues** - Someone might already be working on it
2. **Open an issue first** - For major changes, discuss the approach
3. **One feature per PR** - Keep pull requests focused

### Development Workflow

1. **Create a branch**
   ```bash
   git checkout -b feature/my-feature
   # or
   git checkout -b fix/my-bugfix
   ```

2. **Make your changes**
   - Write clean, readable code
   - Follow the coding standards (see below)
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**
   ```bash
   # Run tests
   cargo test

   # Test on actual hardware if possible
   sudo cargo run -- init /dev/sdX  # BE CAREFUL!
   ```

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "Add feature: brief description"
   ```

   Use clear commit messages:
   - `Add feature: <description>`
   - `Fix bug: <description>`
   - `Update docs: <description>`
   - `Refactor: <description>`

5. **Push to your fork**
   ```bash
   git push origin feature/my-feature
   ```

6. **Open a Pull Request** on GitHub

## Testing

### Automated Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Manual Testing

See [TESTING.md](TESTING.md) for comprehensive testing procedures.

**Critical**: Always test on:
1. Virtual machines first (VirtualBox with USB passthrough)
2. Spare USB drives (never your main storage!)
3. Different boot modes (UEFI and BIOS)
4. At least 2-3 different ISO types

### Test Checklist

Before submitting a PR, ensure:
- [ ] `cargo test` passes
- [ ] `cargo build --release` succeeds without warnings
- [ ] Code is formatted with `cargo fmt`
- [ ] Code passes `cargo clippy` (no warnings)
- [ ] Tested on actual hardware (if applicable)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for significant changes)

## Submitting Changes

### Pull Request Guidelines

1. **Title**: Clear and descriptive
   - Good: "Add support for Arch Linux ISO detection"
   - Bad: "Update config.rs"

2. **Description**: Include:
   - What problem does this solve?
   - How did you solve it?
   - Any breaking changes?
   - Testing done
   - Related issues (e.g., "Fixes #123")

3. **Code**:
   - Clean, readable, well-commented
   - Follows project structure
   - No unrelated changes

4. **Tests**:
   - Add tests for new features
   - Ensure existing tests pass
   - Update tests if behavior changes

### Review Process

1. Maintainers will review your PR
2. Address any requested changes
3. Once approved, it will be merged
4. Your contribution will be acknowledged!

## Coding Standards

### Rust Style

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy

# Both should pass without warnings
cargo fmt -- --check
cargo clippy -- -D warnings
```

### Code Organization

```rust
// Module structure
pub mod device;
pub mod disk;
// ...

// Imports: group and sort
use std::path::{Path, PathBuf};
use std::fs;
use crate::{Result, MultiBootError};

// Public API first, private helpers after
pub fn public_function() { }

fn private_helper() { }
```

### Error Handling

Use the `Result` type consistently:

```rust
// Good
pub fn my_function() -> Result<()> {
    let device = find_device(path)?;
    // ...
    Ok(())
}

// Avoid
pub fn my_function() -> bool {
    // Returns bool instead of Result
}
```

### Documentation

Document all public functions:

```rust
/// Brief description of what this function does
///
/// More detailed explanation if needed.
///
/// # Arguments
/// * `device` - The device path (e.g., /dev/sdb)
///
/// # Returns
/// Device information struct
///
/// # Errors
/// Returns error if device not found
pub fn get_device_info<P: AsRef<Path>>(device: P) -> Result<DeviceInfo> {
    // ...
}
```

### Safety

This tool modifies disks, so safety is critical:

1. **Always validate** device paths
2. **Check** if device is removable
3. **Confirm** before destructive operations
4. **Handle errors** gracefully
5. **Provide clear error messages**

Example:
```rust
// Good - validates first
pub fn init_device<P: AsRef<Path>>(device: P) -> Result<()> {
    validate_device(&device)?;
    verify_removable(&device)?;
    confirm_operation("This will erase all data!")?;
    // ... proceed
}
```

## Documentation

### Types of Documentation

1. **Code comments** - Explain complex logic
2. **Doc comments** - API documentation (`///`)
3. **README.md** - User guide
4. **ARCHITECTURE.md** - Technical design
5. **TESTING.md** - Testing procedures

### Writing Good Documentation

- **Be clear and concise**
- **Use examples** when helpful
- **Keep it updated** when code changes
- **Explain the "why"** not just the "what"

Example:
```rust
// Bad comment
// Set x to 5
let x = 5;

// Good comment
// Initialize buffer size to 5MB for optimal performance on USB 2.0
let buffer_size = 5 * 1024 * 1024;
```

## Project Structure

```
multi-boot/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── cli.rs            # Command-line interface
│   ├── error.rs          # Error types
│   ├── device/           # Device detection
│   │   ├── mod.rs
│   │   ├── detect.rs
│   │   └── info.rs
│   ├── disk/             # Disk operations
│   │   ├── mod.rs
│   │   ├── partition.rs
│   │   └── filesystem.rs
│   ├── bootloader/       # GRUB installation
│   │   ├── mod.rs
│   │   ├── grub.rs
│   │   └── config.rs
│   ├── iso/              # ISO management
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   └── detector.rs
│   └── config/           # Configuration
│       ├── mod.rs
│       └── settings.rs
├── tests/                # Integration tests
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
├── TESTING.md
└── CONTRIBUTING.md
```

## Common Tasks

### Adding Support for a New Distribution

1. **Update ISO detection** in `src/bootloader/config.rs`:
   ```rust
   fn detect_iso_type(filename: &str) -> IsoType {
       let lower = filename.to_lowercase();

       if lower.contains("your-distro") {
           IsoType::YourDistro
       }
       // ...
   }
   ```

2. **Add boot entry generator**:
   ```rust
   fn generate_yourdistro_entry(&self, entry: &IsoEntry) -> String {
       format!(
           "menuentry \"{}\" {{\n\
           \tset isofile=\"/isos/{}\"\n\
           \tloopback loop $isofile\n\
           \tlinux (loop)/path/to/kernel boot=params iso-scan/filename=$isofile\n\
           \tinitrd (loop)/path/to/initrd\n\
           }}",
           entry.name, entry.filename
       )
   }
   ```

3. **Test thoroughly** with actual ISOs

### Adding a New Command

1. **Update CLI** in `src/cli.rs`:
   ```rust
   #[derive(Subcommand)]
   pub enum Commands {
       // ... existing commands

       /// New command description
       NewCommand {
           /// Argument description
           #[arg(short, long)]
           option: String,
       },
   }
   ```

2. **Implement handler** in `src/main.rs`:
   ```rust
   fn cmd_newcommand(option: &str) -> Result<()> {
       // Implementation
       Ok(())
   }
   ```

3. **Add to match** in `run()`:
   ```rust
   Commands::NewCommand { option } => {
       cmd_newcommand(option)
   }
   ```

## Getting Help

- **Documentation**: Read README.md, ARCHITECTURE.md, and code comments
- **Issues**: Search existing issues on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Code**: Read existing code for patterns and examples

## Recognition

All contributors will be recognized in:
- GitHub contributors list
- CONTRIBUTORS.md file (if we create one)
- Release notes for significant contributions

## Thank You!

Your contributions make this project better for everyone. Whether it's code, documentation, testing, or feedback - it all helps!

Happy coding! 🚀
