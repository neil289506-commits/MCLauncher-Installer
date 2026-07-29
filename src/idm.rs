//! Internet Download Manager (IDM) detection + command-line integration.
//! IDM command-line switches (official docs: <https://www.internetdownloadmanager.com/support/command_line.html>):
//!   /d URL      queue a download URL
//!   /p path     target save folder
//!   /f name     target save filename
//!   /n          silent mode (no confirmation dialog, starts immediately)

use std::path::PathBuf;
use std::process::Command;

/// Try to detect IDM via the registry and common install paths.
pub fn detect_idm() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let candidates = [
            (HKEY_CURRENT_USER, r"Software\DownloadManager"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\DownloadManager"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Internet Download Manager"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Internet Download Manager"),
        ];

        for (hive, path) in candidates {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey(path) {
                if let Ok(exe_path) = key.get_value::<String, _>("ExePath") {
                    let p = PathBuf::from(exe_path);
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    let fallback_paths = [
        r"C:\Program Files (x86)\Internet Download Manager\IDMan.exe",
        r"C:\Program Files\Internet Download Manager\IDMan.exe",
    ];
    fallback_paths
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// Call the IDM command line to queue a download and start it silently.
pub fn download_with_idm(
    idm_exe: &PathBuf,
    url: &str,
    save_dir: &str,
    filename: &str,
) -> std::io::Result<()> {
    Command::new(idm_exe)
        .args(["/d", url, "/p", save_dir, "/f", filename, "/n"])
        .spawn()?;
    Ok(())
}
