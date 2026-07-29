//! Minimal, dependency-free i18n: a `Lang` switch (English / Simplified
//! Chinese) plus a static `Strings` table for each language. No external
//! localization crate is used — just two `const` tables of `&'static str`,
//! selected at render time based on the current `Lang`.

#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    En,
    ZhHans,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::En => Lang::ZhHans,
            Lang::ZhHans => Lang::En,
        }
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::ZhHans => &ZH_HANS,
        }
    }
}

pub struct Strings {
    pub pick_launcher: &'static str,
    pub tag_both: &'static str,
    pub tag_installer: &'static str,
    pub tag_portable: &'static str,
    pub tag_none: &'static str,
    pub idm_prefix: &'static str,
    pub idm_not_found: &'static str,
    pub back: &'static str,
    pub installer_label: &'static str,
    pub portable_label: &'static str,
    pub download_btn: &'static str,
    pub not_available: &'static str,
    pub save_location: &'static str,
    pub downloading: &'static str,
    pub merging: &'static str,
    pub fetching_info: &'static str,
    pub download_failed: &'static str,
    pub download_complete: &'static str,
    pub back_to_list: &'static str,
    pub handed_to_idm: &'static str,
    pub handed_to_idm_manual_suffix: &'static str,
    pub idm_launch_failed: &'static str,
    pub installer_launched: &'static str,
    pub installer_launch_failed: &'static str,
}

pub static EN: Strings = Strings {
    pick_launcher: "Pick a launcher to see its available downloads",
    tag_both: "Installer + Portable",
    tag_installer: "Installer",
    tag_portable: "Portable",
    tag_none: "—",
    idm_prefix: "IDM detected ✔ — download via:",
    idm_not_found: "IDM not found — using built-in downloader",
    back: "←  Back",
    installer_label: "Installer",
    portable_label: "Portable",
    download_btn: "Download",
    not_available: "Not available",
    save_location: "Save location: ",
    downloading: "Downloading ",
    merging: "Merging segments…",
    fetching_info: "Fetching file info… (preparing multi-threaded curl download)",
    download_failed: "Download failed: ",
    download_complete: "Download complete!",
    back_to_list: "Back to list",
    handed_to_idm: "Handed off to IDM: ",
    handed_to_idm_manual_suffix: " (run it yourself once IDM finishes)",
    idm_launch_failed: "Failed to launch IDM: ",
    installer_launched: "Installer launched",
    installer_launch_failed: "Failed to launch installer: ",
};

pub static ZH_HANS: Strings = Strings {
    pick_launcher: "选择一个启动器以查看可下载版本",
    tag_both: "安装版 + 可携版",
    tag_installer: "安装版",
    tag_portable: "可携版",
    tag_none: "—",
    idm_prefix: "已检测到 IDM ✔ — 下载方式：",
    idm_not_found: "未检测到 IDM，将使用内置下载器",
    back: "←  返回",
    installer_label: "安装版",
    portable_label: "可携版",
    download_btn: "下载",
    not_available: "不提供",
    save_location: "下载保存位置：",
    downloading: "正在下载 ",
    merging: "正在合并分段文件…",
    fetching_info: "正在获取文件信息…（准备多线程 curl 下载）",
    download_failed: "下载失败：",
    download_complete: "下载完成！",
    back_to_list: "返回列表",
    handed_to_idm: "已交给 IDM 下载：",
    handed_to_idm_manual_suffix: "（IDM 下载完成后请自行运行）",
    idm_launch_failed: "呼叫 IDM 失败：",
    installer_launched: "已启动安装程序",
    installer_launch_failed: "启动安装程序失败：",
};
