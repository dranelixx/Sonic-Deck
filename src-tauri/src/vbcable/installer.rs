//! VB-Cable installer download and launch
//!
//! Provides functionality to download, extract, and launch the VB-Cable installer.

use std::fs::{self, File};
use std::io::{copy, Cursor};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{WaitForSingleObject, INFINITE};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

const VBCABLE_DOWNLOAD_URL: &str =
    "https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack45.zip";
const VBCABLE_ZIP_NAME: &str = "VBCABLE_Driver_Pack45.zip";
const VBCABLE_INSTALLER_NAME: &str = "VBCABLE_Setup_x64.exe";

/// Download VB-Cable installer ZIP to temp directory
pub fn download_vbcable() -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir().join("sonicdeck_vbcable");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let zip_path = temp_dir.join(VBCABLE_ZIP_NAME);

    info!("Downloading VB-Cable from {}", VBCABLE_DOWNLOAD_URL);

    let response = reqwest::blocking::get(VBCABLE_DOWNLOAD_URL)
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        error!(
            "VB-Cable download failed with status: {}",
            response.status()
        );
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;
    let mut file = File::create(&zip_path).map_err(|e| format!("Failed to create file: {}", e))?;
    copy(&mut Cursor::new(bytes), &mut file).map_err(|e| format!("Failed to write file: {}", e))?;

    info!("Downloaded VB-Cable ZIP to {:?}", zip_path);
    Ok(zip_path)
}

/// Extract ALL files from ZIP (installer needs .inf, .sys, .cat files)
pub fn extract_installer(zip_path: &PathBuf) -> Result<PathBuf, String> {
    let temp_dir = zip_path.parent().ok_or("Invalid zip path")?;

    let file = File::open(zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;

    info!("Extracting {} files from VB-Cable ZIP...", archive.len());

    let mut installer_path: Option<PathBuf> = None;

    // Extract ALL files from the archive
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

        // Get the file name (strip any directory prefix)
        let file_name = match file.enclosed_name() {
            Some(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .ok_or_else(|| format!("Invalid file name in ZIP entry {}", i))?,
            None => continue, // Skip entries without valid names (e.g., directories)
        };

        let out_path = temp_dir.join(&file_name);

        // Create the output file
        let mut outfile = File::create(&out_path)
            .map_err(|e| format!("Failed to create {}: {}", file_name, e))?;
        copy(&mut file, &mut outfile)
            .map_err(|e| format!("Failed to extract {}: {}", file_name, e))?;

        debug!("Extracted: {}", file_name);

        // Track the installer path
        if file_name == VBCABLE_INSTALLER_NAME {
            installer_path = Some(out_path);
        }
    }

    match installer_path {
        Some(path) => {
            info!("Extracted {} files, installer at {:?}", archive.len(), path);
            Ok(path)
        }
        None => {
            error!(
                "Installer {} not found in ZIP archive",
                VBCABLE_INSTALLER_NAME
            );
            Err("Installer not found in ZIP".to_string())
        }
    }
}

/// Launch VB-Cable installer with admin elevation (UAC prompt) and wait for completion
///
/// Uses ShellExecuteExW with "runas" verb to request elevation.
/// Waits for the installer to complete before returning.
pub fn launch_installer(installer_path: &PathBuf) -> Result<(), String> {
    info!(
        "Launching VB-Cable installer with elevation: {:?}",
        installer_path
    );

    // Convert strings to wide strings for Windows API
    let operation: Vec<u16> = "runas\0".encode_utf16().collect();
    let file: Vec<u16> = installer_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // Flags: -i (install), -h (headless/silent)
    let parameters: Vec<u16> = "-i -h\0".encode_utf16().collect();
    let directory: Vec<u16> = installer_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    debug!(
        "ShellExecuteExW: operation=runas, file={:?}, params=-i -h",
        installer_path
    );

    // Setup SHELLEXECUTEINFOW structure
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS, // Keep process handle so we can wait
        hwnd: HWND::default(),
        lpVerb: PCWSTR::from_raw(operation.as_ptr()),
        lpFile: PCWSTR::from_raw(file.as_ptr()),
        lpParameters: PCWSTR::from_raw(parameters.as_ptr()),
        lpDirectory: PCWSTR::from_raw(directory.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    // Launch with elevation
    let result = unsafe { ShellExecuteExW(&mut sei) };

    if let Err(e) = result {
        error!("ShellExecuteExW failed: {}", e);
        return Err(format!("Failed to launch installer: {}", e));
    }

    // Wait for the installer to complete
    if !sei.hProcess.is_invalid() {
        info!("Waiting for VB-Cable installer to complete...");

        let wait_result = unsafe { WaitForSingleObject(sei.hProcess, INFINITE) };

        if wait_result == WAIT_OBJECT_0 {
            info!("VB-Cable installer completed");
        } else {
            warn!("WaitForSingleObject returned: {:?}", wait_result);
        }

        // Close the process handle
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(sei.hProcess);
        }
        Ok(())
    } else {
        // User cancelled UAC prompt or installer failed to start
        error!("No process handle - installation cancelled or failed to start");
        Err("Installation cancelled or failed to start".to_string())
    }
}

/// Full installation flow: download, extract, launch
pub fn install_vbcable() -> Result<(), String> {
    info!("Starting VB-Cable installation flow");

    let zip_path = download_vbcable()?;
    let installer_path = extract_installer(&zip_path)?;
    launch_installer(&installer_path)?;

    // Cleanup ZIP (keep installer in case user needs to retry)
    if let Err(e) = fs::remove_file(&zip_path) {
        warn!("Failed to cleanup ZIP file: {}", e);
    }

    info!("VB-Cable installation flow completed");
    Ok(())
}

/// Cleanup temp files
pub fn cleanup_temp_files() {
    let temp_dir = std::env::temp_dir().join("sonicdeck_vbcable");
    if temp_dir.exists() {
        match fs::remove_dir_all(&temp_dir) {
            Ok(_) => info!("Cleaned up VB-Cable temp files"),
            Err(e) => warn!("Failed to cleanup temp files: {}", e),
        }
    }
}

/// Launch VB-Cable uninstaller with admin elevation (UAC prompt) and wait for completion
///
/// Uses the same installer executable with -u flag for uninstall.
fn launch_uninstaller(installer_path: &PathBuf) -> Result<(), String> {
    info!(
        "Launching VB-Cable uninstaller with elevation: {:?}",
        installer_path
    );

    // Convert strings to wide strings for Windows API
    let operation: Vec<u16> = "runas\0".encode_utf16().collect();
    let file: Vec<u16> = installer_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // Flags: -u (uninstall), -h (headless/silent)
    let parameters: Vec<u16> = "-u -h\0".encode_utf16().collect();
    let directory: Vec<u16> = installer_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    debug!(
        "ShellExecuteExW: operation=runas, file={:?}, params=-u -h",
        installer_path
    );

    // Setup SHELLEXECUTEINFOW structure
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS, // Keep process handle so we can wait
        hwnd: HWND::default(),
        lpVerb: PCWSTR::from_raw(operation.as_ptr()),
        lpFile: PCWSTR::from_raw(file.as_ptr()),
        lpParameters: PCWSTR::from_raw(parameters.as_ptr()),
        lpDirectory: PCWSTR::from_raw(directory.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    // Launch with elevation
    let result = unsafe { ShellExecuteExW(&mut sei) };

    if let Err(e) = result {
        error!("ShellExecuteExW failed: {}", e);
        return Err(format!("Failed to launch uninstaller: {}", e));
    }

    // Wait for the uninstaller to complete
    if !sei.hProcess.is_invalid() {
        info!("Waiting for VB-Cable uninstaller to complete...");

        let wait_result = unsafe { WaitForSingleObject(sei.hProcess, INFINITE) };

        if wait_result == WAIT_OBJECT_0 {
            info!("VB-Cable uninstaller completed");
        } else {
            warn!("WaitForSingleObject returned: {:?}", wait_result);
        }

        // Close the process handle
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(sei.hProcess);
        }
    } else {
        warn!("No process handle returned - uninstaller may have failed to start");
    }

    Ok(())
}

/// Full uninstallation flow: download installer (if needed), extract, launch with -u flag
pub fn uninstall_vbcable() -> Result<(), String> {
    info!("Starting VB-Cable uninstallation flow");

    // Check if installer already exists in temp directory
    let temp_dir = std::env::temp_dir().join("sonicdeck_vbcable");
    let installer_path = temp_dir.join(VBCABLE_INSTALLER_NAME);

    let final_installer_path = if installer_path.exists() {
        info!("Using existing installer at {:?}", installer_path);
        installer_path
    } else {
        // Need to download the installer first
        info!("Installer not found, downloading...");
        let zip_path = download_vbcable()?;
        let path = extract_installer(&zip_path)?;

        // Cleanup ZIP
        if let Err(e) = fs::remove_file(&zip_path) {
            warn!("Failed to cleanup ZIP file: {}", e);
        }

        path
    };

    launch_uninstaller(&final_installer_path)?;

    info!("VB-Cable uninstallation flow completed");
    Ok(())
}
