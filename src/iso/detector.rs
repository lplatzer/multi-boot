use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct IsoInfo {
    pub path: PathBuf,
    pub filename: String,
    pub size: u64,
    pub size_human: String,
}

impl IsoInfo {
    pub fn from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        let size = metadata.len();

        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.iso")
            .to_string();

        Ok(Self {
            path: path.to_path_buf(),
            filename,
            size,
            size_human: format_bytes(size),
        })
    }

    pub fn validate(&self) -> bool {
        // Check if file exists
        if !self.path.exists() {
            return false;
        }

        // Check if it's a file (not a directory)
        if !self.path.is_file() {
            return false;
        }

        // Check extension
        if self.path.extension().and_then(|s| s.to_str()) != Some("iso") {
            return false;
        }

        // Basic size check (ISOs should be at least 1MB)
        if self.size < 1_000_000 {
            return false;
        }

        true
    }
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
