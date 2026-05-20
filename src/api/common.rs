// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::api::client::WslUiClient;
use tracing::{debug, error};
use chrono;

use crate::api::models::*;

// Get wslui remote time
pub fn wslui_standard_time() -> i64 {
    let client = WslUiClient::new();
    match client.request_api1::<TimeData>("GET", "/common/v1/utils/time", None) {
        Ok((resp, date_header)) => {
            if resp.data.unix_time > 0 {
                debug!("Obtained time from wslui JSON: {}", resp.data.unix_time);
                return resp.data.unix_time;
            }
            
            // Fallback to Date header if unix_time <= 0
            if let Some(date_str) = date_header {
                match chrono::DateTime::parse_from_rfc2822(&date_str) {
                    Ok(dt) => {
                        let ts = dt.timestamp_millis();
                        debug!("Obtained time from wslui Date header: {}", ts);
                        return ts;
                    }
                    Err(e) => {
                        error!("Failed to parse Date header from wslui: {}", e);
                    }
                }
            }
            0
        }
        Err(e) => {
            error!("Failed to get time from wslui: {}", e);
            0
        }
    }
}

// Get wslui latest release version
pub fn wslui_latest_release() -> ReleaseData {
    let client = WslUiClient::new();
    let fallback = ReleaseData::default();

    match client.request_api2_with_timeout::<ReleaseData>("GET", "/common/v1/releases/latest", None, Some(10)) {
        Ok((resp, _)) => {
            // client.request_api1 internally checks resp.err != 0, so if we reach here, err == 0
            debug!("Obtained latest release from wslui: {:?}", resp.data);
            resp.data
        }
        Err(e) => {
            error!("Failed to get latest release from wslui: {}. Using fallback data.", e);
            fallback
        }
    }
}

// Get helper about information
pub fn wslui_helper_about() -> HelperAboutData {
    let client = WslUiClient::new();
    match client.request_api1::<HelperAboutData>("GET", "/desktop/v1/helper/about", None) {
        Ok((resp, _)) => {
            debug!("Obtained helper about from wslui: {:?}", resp.data);
            resp.data
        }
        Err(e) => {
            error!("Failed to get helper about from wslui: {}. Using default data.", e);
            HelperAboutData::default()
        }
    }
}

// Get helper distro information
pub fn wslui_helper_distro() -> HelperDistroData {
    let client = WslUiClient::new();
    match client.request_api1::<HelperDistroData>("GET", "/desktop/v1/helper/distro", None) {
        Ok((resp, _)) => {
            debug!("Obtained helper distro from wslui: {:?}", resp.data);
            resp.data
        }
        Err(e) => {
            error!("Failed to get helper distro from wslui: {}. Using default data.", e);
            HelperDistroData::default()
        }
    }
}

// Get helper installation information
pub fn wslui_helper_install() -> HelperInstallData {
    let client = WslUiClient::new();
    match client.request_api1::<HelperInstallData>("GET", "/desktop/v1/helper/install", None) {
        Ok((resp, _)) => {
            debug!("Obtained helper install from wslui: {:?}", resp.data);
            resp.data
        }
        Err(e) => {
            error!("Failed to get helper install from wslui: {}. Using default data.", e);
            HelperInstallData::default()
        }
    }
}
