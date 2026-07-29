// Build-time script: embeds assets/icon.ico into the Windows exe resources,
// so File Explorer / taskbar shows a custom ".exe file icon" too.
// (The runtime window/taskbar "window icon" is set separately in main.rs by
// embedding assets/icon.png via include_bytes! + the image crate — a different
// mechanism from the exe's own file icon.)

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            // If rc.exe (normally installed alongside the Windows SDK / MSVC
            // toolchain) or the icon file is missing, don't fail the whole
            // build — just print a warning and continue without an exe icon.
            println!("cargo:warning=Could not embed exe icon: {e}");
        }
    }
}
