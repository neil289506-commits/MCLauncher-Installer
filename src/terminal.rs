//! A minimal Win32 console ("terminal window") helper.
//!
//! When the user downloads via curl (instead of IDM), we pop up a real
//! console window and print progress/status lines to it, so the download
//! is visibly happening in a terminal — similar in spirit to how a CLI
//! download tool would look.
//!
//! Implemented with raw `kernel32` FFI (no extra crate needed): `AllocConsole`
//! creates a new console window attached to this (GUI-subsystem) process,
//! and `WriteConsoleW` writes UTF-16 text straight to it. The console is
//! closed automatically (`FreeConsole`, via `Drop`) once the caller drops
//! the `Terminal` — the app deliberately does NOT rely on the user closing
//! the popped-up window themselves, since closing a process's *only*
//! console via its `X` button can terminate the whole process.

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;
    use std::ptr;

    #[link(name = "kernel32")]
    extern "system" {
        fn AllocConsole() -> i32;
        fn FreeConsole() -> i32;
        fn GetStdHandle(std_handle: i32) -> *mut c_void;
        fn SetConsoleTitleW(title: *const u16) -> i32;
        fn WriteConsoleW(
            console_output: *mut c_void,
            buffer: *const u16,
            chars_to_write: u32,
            chars_written: *mut u32,
            reserved: *mut c_void,
        ) -> i32;
    }

    const STD_OUTPUT_HANDLE: i32 = -11;

    pub struct Terminal {
        handle: *mut c_void,
    }

    impl Terminal {
        /// Opens a new console window with the given title. Returns `None`
        /// if a console could not be allocated (e.g. one already exists).
        pub fn open(title: &str) -> Option<Self> {
            unsafe {
                if AllocConsole() == 0 {
                    return None;
                }
                let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                SetConsoleTitleW(wide_title.as_ptr());
                let handle = GetStdHandle(STD_OUTPUT_HANDLE);
                if handle.is_null() {
                    return None;
                }
                Some(Self { handle })
            }
        }

        /// Writes one line of text (a trailing CRLF is added automatically).
        pub fn write_line(&self, text: &str) {
            unsafe {
                let wide: Vec<u16> = text
                    .encode_utf16()
                    .chain([b'\r' as u16, b'\n' as u16])
                    .collect();
                let mut written: u32 = 0;
                WriteConsoleW(
                    self.handle,
                    wide.as_ptr(),
                    wide.len() as u32,
                    &mut written,
                    ptr::null_mut(),
                );
            }
        }
    }

    impl Drop for Terminal {
        fn drop(&mut self) {
            unsafe {
                FreeConsole();
            }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub struct Terminal;

    impl Terminal {
        pub fn open(_title: &str) -> Option<Self> {
            None
        }

        pub fn write_line(&self, _text: &str) {}
    }
}

pub use imp::Terminal;
