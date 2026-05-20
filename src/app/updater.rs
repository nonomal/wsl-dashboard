// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use semver::Version;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub has_update: bool,
    pub latest_version: String,
    pub release_date: String,
    pub current_version: String,
    pub download_url: String,
    pub error: Option<String>,
}

pub async fn check_update(current_version_str: &str) -> Result<UpdateResult, String> {
    let current_version_str = current_version_str.to_string();

    // Use wslui_latest_release instead of fetch_base_config
    let release_data = tokio::task::spawn_blocking(move || {
        crate::api::common::wslui_latest_release()
    }).await.map_err(|e| format!("Task panicked: {}", e))?;

    let latest_version_str = release_data.version;
    let download_url = release_data.download_url;

    // Version comparison
    let current_v_clean = current_version_str.trim_start_matches('v');
    let latest_v_clean = latest_version_str.trim_start_matches('v');

    let current = Version::parse(current_v_clean)
        .map_err(|e| format!("Failed to parse current version {}: {}", current_version_str, e))?;
    let latest = Version::parse(latest_v_clean)
        .map_err(|e| format!("Failed to parse latest version {}: {}", latest_version_str, e))?;

    info!("Update check: current={}, latest={}", current_version_str, latest_version_str);

    Ok(UpdateResult {
        has_update: latest > current,
        latest_version: latest_version_str,
        release_date: release_data.release_date,
        current_version: current_version_str,
        download_url,
        error: None,
    })
}
