//! Launcher data table: compiled from the user-provided txt list,
//! organized by "Installer / Portable" download links for each launcher.

#[derive(Clone, Copy)]
pub struct LauncherEntry {
    pub name: &'static str,
    pub installer: Option<&'static str>,
    pub portable: Option<&'static str>,
    /// Explicit save filename to use for the installer download, overriding
    /// the URL-guessed name. Needed for links like aka.ms's redirect that
    /// don't have a filename (or extension) anywhere in the URL itself.
    pub installer_filename: Option<&'static str>,
}

pub static LAUNCHERS: &[LauncherEntry] = &[
    LauncherEntry {
        name: "Minecraft Launcher (Official)",
        installer: Some("https://aka.ms/minecraftClientGameCoreWindows"),
        portable: None,
        installer_filename: Some("MinecraftInstaller.exe"),
    },
    LauncherEntry {
        name: "PrismLauncher",
        installer: Some("https://github.com/PrismLauncher/PrismLauncher/releases/download/11.0.3/PrismLauncher-Windows-MSVC-Setup-11.0.3.exe"),
        portable: Some("https://github.com/PrismLauncher/PrismLauncher/releases/download/11.0.3/PrismLauncher-Windows-MinGW-w64-Portable-11.0.3.zip"),
        installer_filename: None,
    },
    LauncherEntry {
        name: "Modrinth App",
        installer: Some("https://launcher-files.modrinth.com/versions/0.16.1/windows/Modrinth%20App_0.16.1_x64-setup.exe"),
        portable: None,
        installer_filename: None,
    },
    LauncherEntry {
        name: "CurseForge",
        installer: Some("https://download.overwolf.com/install/Download?ExtensionId=cfiahnpaolfnlgaihhmobmnjdafknjnjdpdabpcm&utm_term=eyJkb21haW4iOiJjZi13ZWIifQ%3D%3D"),
        portable: None,
        installer_filename: Some("CurseForge-Setup.exe"),
    },
    LauncherEntry {
        name: "XMCL",
        installer: Some("https://xmcl.blob.core.windows.net/releases/xmcl.appinstaller"),
        portable: Some("https://github.com/Voxelum/x-minecraft-launcher/releases/download/v0.65.1/xmcl-0.65.1-win32-x64.zip"),
        installer_filename: None,
    },
    LauncherEntry {
        name: "ATLauncher",
        installer: Some("https://download.nodecdn.net/containers/atl/app/dist/windows/ATLauncher-setup-1.3.0.0.exe"),
        portable: Some("https://download.nodecdn.net/containers/atl/ATLauncher.exe"),
        installer_filename: None,
    },
    LauncherEntry {
        name: "GDLauncher",
        installer: Some("https://gdlauncher.com/download/windows"),
        portable: None,
        installer_filename: Some("GDLauncher-Setup.exe"),
    },
    LauncherEntry {
        name: "LabyMod",
        installer: Some("https://releases.r2.labymod.net/launcher/win32/x64/LabyModLauncherSetup-latest.exe"),
        portable: None,
        installer_filename: None,
    },
    LauncherEntry {
        name: "Lunar Client",
        installer: Some("https://api.lunarclientprod.com/site/download?utm_source=client-site&utm_medium=download-page&utm_campaign=direct&os=windows"),
        portable: None,
        installer_filename: Some("LunarClient-Setup.exe"),
    },
    LauncherEntry {
        name: "Emerald Launcher (LCE)",
        installer: Some("https://github.com/LCE-Hub/LCE-Emerald-Launcher/releases/download/v1.5.1/LCE.Emerald.Launcher_1.5.1_x64-setup.exe"),
        portable: None,
        installer_filename: None,
    },
    LauncherEntry {
        name: "MultiMC",
        installer: None,
        portable: Some("https://files.multimc.org/downloads/mmc-develop-win32.zip"),
        installer_filename: None,
    },
];

/// Guess a save filename from a URL (strip the query string, take the last
/// path segment). If the result has no file extension at all (common for
/// redirect/trigger links like aka.ms or a download-page URL), fall back to
/// appending `.exe`, since every download in this app is a Windows installer
/// or archive — an extensionless file is never the intended outcome.
pub fn guess_filename(url: &str) -> String {
    let clean = url.split('?').next().unwrap_or(url);
    let name = clean.rsplit('/').next().unwrap_or("");
    let mut name = if name.is_empty() { "download".to_string() } else { name.to_string() };
    if !name.contains('.') {
        name.push_str(".exe");
    }
    name
}
