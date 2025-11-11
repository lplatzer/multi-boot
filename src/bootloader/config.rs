use crate::Result;
use std::path::Path;
use std::fs;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct IsoEntry {
    pub filename: String,
    pub name: String,
    pub size: u64,
    pub iso_type: IsoType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IsoType {
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    Generic,
    Windows,
    Unknown,
}

pub struct GrubConfig {
    pub entries: Vec<IsoEntry>,
}

impl GrubConfig {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: IsoEntry) {
        self.entries.push(entry);
    }

    /// Generate GRUB configuration file content
    pub fn generate(&self) -> String {
        let mut config = String::new();

        // GRUB settings
        config.push_str("# Multi-Boot USB - GRUB Configuration\n");
        config.push_str("# Auto-generated - do not edit manually\n\n");

        config.push_str("set timeout=30\n");
        config.push_str("set default=0\n\n");

        config.push_str("# Set root to data partition\n");
        config.push_str("# GRUB will search for the partition with label MULTIBOOT\n");
        config.push_str("search --no-floppy --set=root --label MULTIBOOT\n\n");

        config.push_str("# Load additional modules\n");
        config.push_str("insmod all_video\n");
        config.push_str("insmod gfxterm\n");
        config.push_str("insmod loopback\n");
        config.push_str("insmod iso9660\n");
        config.push_str("insmod exfat\n");
        config.push_str("insmod fat\n");
        config.push_str("insmod part_gpt\n");
        config.push_str("insmod part_msdos\n\n");

        config.push_str("# Set graphical terminal\n");
        config.push_str("terminal_output gfxterm\n\n");

        // Generate entries for each ISO
        if self.entries.is_empty() {
            config.push_str("menuentry \"No ISOs found\" {\n");
            config.push_str("    echo \"No ISO files found in /isos directory\"\n");
            config.push_str("    echo \"Please add ISOs using: multi-boot add <device> <iso>\"\n");
            config.push_str("    echo \"Press any key to reboot...\"\n");
            config.push_str("    read\n");
            config.push_str("    reboot\n");
            config.push_str("}\n\n");
        } else {
            for entry in &self.entries {
                config.push_str(&self.generate_entry(entry));
                config.push_str("\n");
            }
        }

        // Add utility entries
        config.push_str(&self.generate_utility_entries());

        config
    }

    fn generate_entry(&self, entry: &IsoEntry) -> String {
        match entry.iso_type {
            IsoType::Ubuntu | IsoType::Debian => self.generate_ubuntu_entry(entry),
            IsoType::Fedora => self.generate_fedora_entry(entry),
            IsoType::Arch => self.generate_arch_entry(entry),
            IsoType::Generic => self.generate_generic_linux_entry(entry),
            IsoType::Windows => self.generate_windows_entry(entry),
            IsoType::Unknown => self.generate_fallback_entry(entry),
        }
    }

    fn generate_ubuntu_entry(&self, entry: &IsoEntry) -> String {
        format!(
            "menuentry \"{}\" {{\n\
            \tset isofile=\"/isos/{}\"\n\
            \tloopback loop $isofile\n\
            \tlinux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile quiet splash noprompt --\n\
            \tinitrd (loop)/casper/initrd\n\
            }}",
            entry.name, entry.filename
        )
    }

    fn generate_fedora_entry(&self, entry: &IsoEntry) -> String {
        format!(
            "menuentry \"{}\" {{\n\
            \tset isofile=\"/isos/{}\"\n\
            \tloopback loop $isofile\n\
            \tlinux (loop)/isolinux/vmlinuz root=live:CDLABEL=Fedora iso-scan/filename=$isofile rd.live.image quiet\n\
            \tinitrd (loop)/isolinux/initrd.img\n\
            }}",
            entry.name, entry.filename
        )
    }

    fn generate_arch_entry(&self, entry: &IsoEntry) -> String {
        format!(
            "menuentry \"{}\" {{\n\
            \tset isofile=\"/isos/{}\"\n\
            \tloopback loop $isofile\n\
            \tlinux (loop)/arch/boot/x86_64/vmlinuz-linux archisobasedir=arch archisolabel=ARCH img_dev=/dev/disk/by-label/MULTIBOOT img_loop=$isofile\n\
            \tinitrd (loop)/arch/boot/x86_64/initramfs-linux.img\n\
            }}",
            entry.name, entry.filename
        )
    }

    fn generate_generic_linux_entry(&self, entry: &IsoEntry) -> String {
        format!(
            "menuentry \"{}\" {{\n\
            \tset isofile=\"/isos/{}\"\n\
            \tloopback loop $isofile\n\
            \tlinux (loop)/casper/vmlinuz boot=casper iso-scan/filename=$isofile quiet splash --\n\
            \tinitrd (loop)/casper/initrd\n\
            }}",
            entry.name, entry.filename
        )
    }

    fn generate_windows_entry(&self, entry: &IsoEntry) -> String {
        format!(
            "menuentry \"{}\" {{\n\
            \tset isofile=\"/isos/{}\"\n\
            \tloopback loop $isofile\n\
            \tchainloader (loop)/bootmgr\n\
            }}",
            entry.name, entry.filename
        )
    }

    fn generate_fallback_entry(&self, entry: &IsoEntry) -> String {
        format!(
            "menuentry \"{}\" {{\n\
            \tset isofile=\"/isos/{}\"\n\
            \tloopback loop $isofile\n\
            \tchainloader (loop)+1\n\
            }}",
            entry.name, entry.filename
        )
    }

    fn generate_utility_entries(&self) -> String {
        "menuentry \"Reboot\" {\n\
        \treboot\n\
        }\n\n\
        menuentry \"Shutdown\" {\n\
        \thalt\n\
        }\n".to_string()
    }
}

/// Generate GRUB configuration from ISOs in a directory
pub fn generate_grub_config<P: AsRef<Path>>(
    isos_dir: P,
    output_path: P,
) -> Result<()> {
    let isos_path = isos_dir.as_ref();

    println!("Scanning for ISOs in {}...", isos_path.display());

    let mut config = GrubConfig::new();

    // Scan for ISO files
    if isos_path.exists() {
        for entry in fs::read_dir(isos_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("iso") {
                let filename = path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown.iso")
                    .to_string();

                let metadata = entry.metadata()?;
                let size = metadata.len();

                // Detect ISO type from filename
                let iso_type = detect_iso_type(&filename);
                let name = generate_display_name(&filename);

                println!("  Found: {}", name);

                config.add_entry(IsoEntry {
                    filename,
                    name,
                    size,
                    iso_type,
                });
            }
        }
    }

    println!("Generating GRUB configuration...");

    // Write configuration
    let grub_cfg = config.generate();
    let output = output_path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(output)?;
    file.write_all(grub_cfg.as_bytes())?;

    println!("GRUB configuration written to {}", output.display());
    println!("Total ISOs: {}", config.entries.len());

    Ok(())
}

fn detect_iso_type(filename: &str) -> IsoType {
    let lower = filename.to_lowercase();

    if lower.contains("ubuntu") || lower.contains("kubuntu") ||
       lower.contains("xubuntu") || lower.contains("lubuntu") {
        IsoType::Ubuntu
    } else if lower.contains("debian") || lower.contains("mint") {
        IsoType::Debian
    } else if lower.contains("fedora") {
        IsoType::Fedora
    } else if lower.contains("arch") || lower.contains("manjaro") {
        IsoType::Arch
    } else if lower.contains("windows") || lower.contains("win10") ||
              lower.contains("win11") {
        IsoType::Windows
    } else if lower.contains("linux") || lower.contains("centos") ||
              lower.contains("rocky") || lower.contains("alma") {
        IsoType::Generic
    } else {
        IsoType::Unknown
    }
}

fn generate_display_name(filename: &str) -> String {
    // Remove .iso extension
    let name = filename.strip_suffix(".iso").unwrap_or(filename);

    // Replace hyphens and underscores with spaces
    let name = name.replace('-', " ").replace('_', " ");

    // Capitalize first letter of each word
    name.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
