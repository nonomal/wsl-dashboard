// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TimeData {
    pub unix_time: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReleaseData {
    pub version: String,
    pub release_date: String,
    pub download_url: String,
}

impl Default for ReleaseData {
    fn default() -> Self {
        Self {
            version: "10.0.0".to_string(),
            release_date: "2100-10-10".to_string(),
            download_url: crate::app::PROJECT_REPOSITORY.to_string(),
        }
    }
}

// 1. wslui_helper_about data structure
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OfficialGroup {
    pub system_language: String,
    pub timezone: String,
    pub name: String,
    pub pic: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HelperAboutLink {
    #[allow(dead_code)]
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HelperAboutData {
    pub official_group: Vec<OfficialGroup>,
    pub documents: Option<HelperAboutLink>,
    pub discussions: Option<HelperAboutLink>,
}

// 2. wslui_helper_distro data structure
#[derive(Debug, Deserialize, Clone)]
pub struct VSCodeExtension {
    pub name: String,
    pub url: String,
}

impl Default for VSCodeExtension {
    fn default() -> Self {
        Self {
            name: "Microsoft WSL(Identifier:ms-vscode-remote.remote-wsl)".to_string(),
            url: crate::app::VSCODE_MARKETPLACE_URL.to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct StartupScript {
    #[allow(dead_code)]
    pub name: String,
    pub url: String,
}

impl Default for StartupScript {
    fn default() -> Self {
        Self {
            name: "Distro startup script".to_string(),
            url: crate::app::PROJECT_DOCS.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CompressScript {
    #[allow(dead_code)]
    pub name: String,
    pub url: String,
}

impl Default for CompressScript {
    fn default() -> Self {
        Self {
            name: "Linux disk cleanup".to_string(),
            url: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HelperDistroData {
    pub vscode_extension: VSCodeExtension,
    pub startup_script: StartupScript,
    pub compress_script: CompressScript,
}

// 3. wslui_helper_install data structure
#[derive(Debug, Deserialize, Clone, Default)]
pub struct RootfsHelp {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HelperInstallData {
    pub rootfs_help: Vec<RootfsHelp>,
}
