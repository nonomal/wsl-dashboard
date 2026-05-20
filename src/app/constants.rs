// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

// Application constants definition
#[allow(dead_code)]
pub const APP_NAME: &str = "WSL Dashboard";
#[allow(dead_code)]
pub const APP_ID: &str = "wsldashboard";
#[allow(dead_code)]
pub const COMPANY_NAME: &str = APP_NAME;
#[allow(dead_code)]
pub const LEGAL_COPYRIGHT: &str = "2026 WSL Dashboard. All rights reserved.";


#[allow(dead_code)]
pub const PROJECT_REPOSITORY: &str = "https://github.com/owu/wsl-dashboard";

#[allow(dead_code)]
pub const PROJECT_WEBSITE: &str = "https://www.wslui.com";

#[allow(dead_code)]
pub const GITHUB_ISSUES: &str = "/issues";

#[allow(dead_code)]
pub const GITHUB_RELEASES: &str = "/releases";

#[allow(dead_code)]
pub const GITHUB_DOMAIN: &str = "https://github.com";

#[allow(dead_code)]
pub const WSL_GITHUB_RELEASES: &str = "https://github.com/microsoft/WSL/releases/latest";

#[allow(dead_code)]
pub const VSCODE_MARKETPLACE_URL: &str = "https://marketplace.visualstudio.com";

#[allow(dead_code)]
pub const PROJECT_DOCS: &str = "https://docs.wslui.com";

#[allow(dead_code)]
pub const API1_URL: &str = "https://api1.wslui.com";

#[allow(dead_code)]
pub const API2_URL: &str = "https://api2.wslui.com";

// Compatibility of Chinese and Japanese character display on Western language operating systems
// Font constants
#[allow(dead_code)]
pub const FONT_ZH: &str = "Microsoft YaHei UI";
#[allow(dead_code)]
pub const FONT_EN_FALLBACK: &str = "Segoe UI, Microsoft YaHei UI";

// Check if a language code represents Chinese
#[allow(dead_code)]
pub fn is_chinese_lang(lang: &str) -> bool {
    lang.to_lowercase().starts_with("zh")
}

// WSL distribution initialization script path
#[allow(dead_code)]
pub const WSL_INIT_SCRIPT: &str = "/etc/init.wsl-dashboard";

