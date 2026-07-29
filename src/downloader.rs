//! Built-in downloader: used when IDM is not installed on the system, or
//! when the user explicitly picks "curl" over IDM in the UI.
//!
//! Uses the external `curl` command with HTTP Range requests to download in
//! parallel segments, with a background thread polling each segment's file
//! size to report progress to the UI. curl.exe ships with Windows 10 1803
//! and later, so no extra install is usually needed.
//!
//! Because this is the "curl" path, a pop-up console window (see
//! `crate::terminal`) is opened for the duration of the download so the
//! user can see it actually happening, similar to a CLI download tool.
//!
//! All curl calls pass `--ssl-no-revoke`: on Windows, curl's Schannel TLS
//! backend does a certificate-revocation check by default, and on some
//! networks (corporate firewalls/proxies, some antivirus SSL scanning)
//! that revocation check itself can't reach its server, which fails the
//! whole TLS handshake with curl error 35 / Windows error 0x80092012
//! ("撤銷功能無法檢查憑證的撤銷") — even though the certificate itself is
//! fine. `--ssl-no-revoke` skips just that revocation check.

use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::terminal::Terminal;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const THREAD_COUNT: u64 = 8;
/// Files smaller than this (bytes) are downloaded single-threaded instead of segmented.
const MULTI_THREAD_THRESHOLD: u64 = 5_000_000;
/// How long the pop-up terminal stays open after the download finishes, so the
/// user has time to read the final status line before it closes itself.
const TERMINAL_LINGER: Duration = Duration::from_millis(1500);

pub struct DownloadProgress {
    pub downloaded: AtomicU64,
    pub total: AtomicU64,
    pub done: AtomicBool,
    pub merging: AtomicBool,
    pub error: Mutex<Option<String>>,
    /// Set once the post-download installer auto-launch has been attempted,
    /// so the UI only tries it a single time.
    pub launched: AtomicBool,
}

impl DownloadProgress {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            downloaded: AtomicU64::new(0),
            total: AtomicU64::new(0),
            done: AtomicBool::new(false),
            merging: AtomicBool::new(false),
            error: Mutex::new(None),
            launched: AtomicBool::new(false),
        })
    }

    pub fn fraction(&self) -> f32 {
        let total = self.total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        (self.downloaded.load(Ordering::Relaxed) as f32 / total as f32).clamp(0.0, 1.0)
    }
}

fn configure(cmd: &mut Command) {
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
}

/// Get the Content-Length via `curl -sIL` (follows redirects automatically).
fn probe_size(url: &str) -> Option<u64> {
    let mut cmd = Command::new("curl");
    cmd.args(["-sIL", "--ssl-no-revoke", url]);
    configure(&mut cmd);
    let output = cmd.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter_map(|l| {
            let lower = l.to_ascii_lowercase();
            lower
                .starts_with("content-length:")
                .then(|| l.split(':').nth(1))
                .flatten()
                .and_then(|v| v.trim().parse::<u64>().ok())
        })
        .last()
}

/// Kick off the download on a background thread; progress is reported live via `progress`.
/// Also pops up a console ("terminal") window for the duration of the download.
pub fn start_download(url: String, save_path: PathBuf, progress: Arc<DownloadProgress>) {
    thread::spawn(move || {
        let terminal = Terminal::open("Minecraft Launcher Installer - curl download");
        let log = |line: &str| {
            if let Some(t) = &terminal {
                t.write_line(line);
            }
        };

        log(&format!("Downloading: {url}"));
        log(&format!("Saving to:   {}", save_path.display()));

        if let Some(parent) = save_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let total = probe_size(&url);
        match total {
            Some(sz) => log(&format!("File size:   {sz} bytes")),
            None => log("File size:   unknown (server didn't report Content-Length)"),
        }

        let tmp_dir = save_path.with_extension("parts_tmp");
        let _ = fs::create_dir_all(&tmp_dir);

        let use_multi = total.map(|t| t > MULTI_THREAD_THRESHOLD).unwrap_or(false);
        if use_multi {
            log(&format!("Starting {THREAD_COUNT} parallel curl segments..."));
        } else {
            log("Starting single-threaded curl download...");
        }

        let result = if use_multi {
            let total = total.unwrap();
            progress.total.store(total, Ordering::Relaxed);
            download_multi(&url, total, &tmp_dir, &save_path, &progress, terminal.as_ref())
        } else {
            download_single(&url, &save_path, &progress, terminal.as_ref())
        };

        let _ = fs::remove_dir_all(&tmp_dir);

        match &result {
            Ok(()) => log("Download complete."),
            Err(e) => log(&format!("Download failed: {e}")),
        }

        if let Err(e) = result {
            *progress.error.lock().unwrap() = Some(e);
        }
        progress.done.store(true, Ordering::Relaxed);

        // Give the user a moment to read the final status before the pop-up
        // terminal window closes itself (see crate::terminal for why we
        // close it ourselves instead of relying on the user clicking X).
        if terminal.is_some() {
            thread::sleep(TERMINAL_LINGER);
        }
    });
}

fn download_multi(
    url: &str,
    total: u64,
    tmp_dir: &Path,
    save_path: &Path,
    progress: &Arc<DownloadProgress>,
    terminal: Option<&Terminal>,
) -> Result<(), String> {
    let chunk = total / THREAD_COUNT;
    let ranges: Vec<(u64, u64)> = (0..THREAD_COUNT)
        .map(|i| {
            let start = i * chunk;
            let end = if i == THREAD_COUNT - 1 { total - 1 } else { start + chunk - 1 };
            (start, end)
        })
        .collect();

    let part_paths: Vec<PathBuf> = (0..THREAD_COUNT)
        .map(|i| tmp_dir.join(format!("part{i}")))
        .collect();

    // Monitor thread: periodically sums up all segment file sizes to report progress.
    let monitor_paths = part_paths.clone();
    let monitor_progress = progress.clone();
    let monitor_stop = Arc::new(AtomicBool::new(false));
    let monitor_stop_clone = monitor_stop.clone();
    let monitor = thread::spawn(move || {
        while !monitor_stop_clone.load(Ordering::Relaxed) {
            let sum: u64 = monitor_paths
                .iter()
                .filter_map(|p| fs::metadata(p).ok())
                .map(|m| m.len())
                .sum();
            monitor_progress.downloaded.store(sum, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(150));
        }
    });

    let handles: Vec<thread::JoinHandle<Result<(), String>>> = ranges
        .into_iter()
        .zip(part_paths.iter().cloned())
        .map(|((start, end), path)| {
            let url = url.to_string();
            thread::spawn(move || -> Result<(), String> {
                let range = format!("{start}-{end}");
                let mut cmd = Command::new("curl");
                cmd.arg("-sL")
                    .arg("--ssl-no-revoke")
                    .arg(&url)
                    .arg("-r")
                    .arg(&range)
                    .arg("-o")
                    .arg(&path)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped());
                configure(&mut cmd);
                let output = cmd.output().map_err(|e| format!("failed to launch curl: {e}"))?;
                if !output.status.success() {
                    return Err(format!(
                        "segment {range} failed: {}",
                        curl_error_message(&output.status, &output.stderr)
                    ));
                }
                Ok(())
            })
        })
        .collect();

    let mut first_err: Option<String> = None;
    for h in handles {
        match h.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                first_err.get_or_insert(e);
            }
            Err(_) => {
                first_err.get_or_insert_with(|| "Download thread encountered an unexpected error".to_string());
            }
        }
    }

    monitor_stop.store(true, Ordering::Relaxed);
    let _ = monitor.join();

    if let Some(e) = first_err {
        return Err(e);
    }

    if let Some(t) = terminal {
        t.write_line("All segments downloaded, merging into final file...");
    }

    progress.merging.store(true, Ordering::Relaxed);
    let merge_result = merge_parts(&part_paths, save_path);
    progress.merging.store(false, Ordering::Relaxed);
    merge_result?;

    if let Some(t) = terminal {
        t.write_line("Merge complete.");
    }

    progress.downloaded.store(total, Ordering::Relaxed);
    Ok(())
}

fn merge_parts(part_paths: &[PathBuf], save_path: &Path) -> Result<(), String> {
    let mut out = File::create(save_path).map_err(|e| e.to_string())?;
    for p in part_paths {
        let f = File::open(p).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(f);
        let mut buf = [0u8; 1 << 20];
        loop {
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn download_single(
    url: &str,
    save_path: &Path,
    progress: &Arc<DownloadProgress>,
    terminal: Option<&Terminal>,
) -> Result<(), String> {
    let mut cmd = Command::new("curl");
    cmd.arg("-sL")
        .arg("--ssl-no-revoke")
        .arg(url)
        .arg("-o")
        .arg(save_path)
        // IMPORTANT: this app runs without a normal inherited console (GUI
        // subsystem in release builds), so its own stdin/stdout/stderr
        // handles can be null/invalid. Without explicitly redirecting the
        // child's stdio here, spawning curl.exe can fail to inherit those
        // handles with "os error 6" (ERROR_INVALID_HANDLE). stderr is piped
        // (not null) so we can surface curl's actual error text instead of
        // just a generic failure message.
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    configure(&mut cmd);
    let output = cmd.output().map_err(|e| format!("failed to launch curl: {e}"))?;
    if !output.status.success() {
        return Err(curl_error_message(&output.status, &output.stderr));
    }
    if let Ok(meta) = fs::metadata(save_path) {
        let len = meta.len().max(1);
        progress.total.store(len, Ordering::Relaxed);
        progress.downloaded.store(len, Ordering::Relaxed);
    }
    if let Some(t) = terminal {
        t.write_line("File saved.");
    }
    Ok(())
}

/// Turns a failed curl invocation's exit status + captured stderr into a
/// readable error string (curl -s suppresses its own progress/errors, so we
/// need -sL + a piped stderr to get anything useful back at all).
fn curl_error_message(status: &std::process::ExitStatus, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    match (status.code(), stderr.is_empty()) {
        (Some(code), false) => format!("curl exited with code {code}: {stderr}"),
        (Some(code), true) => format!("curl exited with code {code}"),
        (None, false) => format!("curl terminated: {stderr}"),
        (None, true) => "curl terminated unexpectedly".to_string(),
    }
}