use crate::{Result, MultiBootError};
use super::detector::IsoInfo;
use std::path::Path;
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};

/// Add an ISO to the USB drive
pub fn add_iso<P1, P2>(
    data_mountpoint: P1,
    iso_path: P2,
    custom_name: Option<&str>,
) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let iso_path = iso_path.as_ref();
    let data_mount = data_mountpoint.as_ref();

    // Validate ISO
    println!("Validating ISO file...");
    let iso_info = IsoInfo::from_path(iso_path)
        .map_err(|e| MultiBootError::IsoNotFound(e.to_string()))?;

    if !iso_info.validate() {
        return Err(MultiBootError::InvalidIso(
            format!("{} is not a valid ISO file", iso_path.display())
        ));
    }

    println!("ISO: {} ({})", iso_info.filename, iso_info.size_human);

    // Create isos directory
    let isos_dir = data_mount.join("isos");
    fs::create_dir_all(&isos_dir)?;

    // Determine destination filename
    let dest_filename = if let Some(name) = custom_name {
        format!("{}.iso", name)
    } else {
        iso_info.filename.clone()
    };

    let dest_path = isos_dir.join(&dest_filename);

    // Check if ISO already exists
    if dest_path.exists() {
        return Err(MultiBootError::Other(
            format!("ISO '{}' already exists on the drive", dest_filename)
        ));
    }

    // Check available space
    let available_space = get_available_space(&data_mount)?;
    if iso_info.size > available_space {
        return Err(MultiBootError::InsufficientSpace {
            needed: iso_info.size,
            available: available_space,
        });
    }

    // Copy ISO with progress bar
    println!("Copying ISO to USB drive...");
    copy_with_progress(&iso_info.path, &dest_path)?;

    println!("ISO added successfully: {}", dest_filename);
    Ok(())
}

/// Remove an ISO from the USB drive
pub fn remove_iso<P>(data_mountpoint: P, iso_name: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let data_mount = data_mountpoint.as_ref();
    let isos_dir = data_mount.join("isos");

    let iso_path = isos_dir.join(iso_name);

    if !iso_path.exists() {
        return Err(MultiBootError::IsoNotFound(
            format!("ISO '{}' not found on the drive", iso_name)
        ));
    }

    println!("Removing ISO: {}", iso_name);
    fs::remove_file(&iso_path)?;

    println!("ISO removed successfully");
    Ok(())
}

/// List all ISOs on the USB drive
pub fn list_isos<P>(data_mountpoint: P, detailed: bool) -> Result<Vec<IsoInfo>>
where
    P: AsRef<Path>,
{
    let data_mount = data_mountpoint.as_ref();
    let isos_dir = data_mount.join("isos");

    if !isos_dir.exists() {
        return Ok(Vec::new());
    }

    let mut isos = Vec::new();

    for entry in fs::read_dir(&isos_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("iso") {
            if let Ok(info) = IsoInfo::from_path(&path) {
                if detailed {
                    println!("  {} - {}", info.filename, info.size_human);
                } else {
                    println!("  {}", info.filename);
                }
                isos.push(info);
            }
        }
    }

    Ok(isos)
}

// Unix-specific implementation using libc
#[cfg(unix)]
fn get_available_space<P: AsRef<Path>>(path: P) -> Result<u64> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let path_cstr = CString::new(path.as_ref().to_str().unwrap())
        .map_err(|e| MultiBootError::Other(e.to_string()))?;

    unsafe {
        let mut stat: MaybeUninit<libc::statfs> = MaybeUninit::uninit();
        let ret = libc::statfs(path_cstr.as_ptr(), stat.as_mut_ptr());

        if ret != 0 {
            return Err(MultiBootError::Other("statfs failed".to_string()));
        }

        let stat = stat.assume_init();
        Ok(stat.f_bavail * stat.f_bsize as u64)
    }
}

// For platforms without Unix statfs, use a different approach
#[cfg(not(unix))]
fn get_available_space<P: AsRef<Path>>(_path: P) -> Result<u64> {
    // Fallback: assume sufficient space
    Ok(u64::MAX)
}

fn copy_with_progress<P1, P2>(src: P1, dest: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let src = src.as_ref();
    let dest = dest.as_ref();

    let src_size = fs::metadata(src)?.len();

    let pb = ProgressBar::new(src_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Use std::fs::copy for simplicity
    // For more control, we could read/write in chunks and update progress
    let mut src_file = fs::File::open(src)?;
    let mut dest_file = fs::File::create(dest)?;

    let buffer_size = 1024 * 1024; // 1MB buffer
    let mut buffer = vec![0u8; buffer_size];
    let mut copied = 0u64;

    loop {
        let n = std::io::Read::read(&mut src_file, &mut buffer)?;
        if n == 0 {
            break;
        }

        std::io::Write::write_all(&mut dest_file, &buffer[..n])?;
        copied += n as u64;
        pb.set_position(copied);
    }

    pb.finish_with_message("Copy complete");

    Ok(())
}

