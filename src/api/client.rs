// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use ureq;
use std::time::Duration;
use std::thread;
use tracing::{debug, error};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub err: i32,
    pub msg: String,
    pub data: T,
}

pub struct WslUiClient {
    api1_url: String,
    api2_url: String,
}

impl WslUiClient {
    pub fn new() -> Self {
        Self {
            api1_url: crate::app::API1_URL.to_string(),
            api2_url: crate::app::API2_URL.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn request_api1<T>(&self, method: &str, uri: &str, body: Option<serde_json::Value>) -> Result<(ApiResponse<T>, Option<String>), String>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        self.request_api1_with_timeout(method, uri, body, None)
    }

    pub fn request_api1_with_timeout<T>(&self, method: &str, uri: &str, body: Option<serde_json::Value>, timeout_secs: Option<u64>) -> Result<(ApiResponse<T>, Option<String>), String>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let url = format!("{}{}", self.api1_url, uri);
        self.request_url(method, &url, body, timeout_secs)
    }

    #[allow(dead_code)]
    pub fn request_api2<T>(&self, method: &str, uri: &str, body: Option<serde_json::Value>) -> Result<(ApiResponse<T>, Option<String>), String>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        self.request_api2_with_timeout(method, uri, body, None)
    }

    pub fn request_api2_with_timeout<T>(&self, method: &str, uri: &str, body: Option<serde_json::Value>, timeout_secs: Option<u64>) -> Result<(ApiResponse<T>, Option<String>), String>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let url = format!("{}{}", self.api2_url, uri);
        self.request_url(method, &url, body, timeout_secs)
    }

    fn request_url<T>(&self, method: &str, url: &str, body: Option<serde_json::Value>, timeout_secs: Option<u64>) -> Result<(ApiResponse<T>, Option<String>), String>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(5));
        let do_request = || -> Result<(ApiResponse<T>, Option<String>), String> {
            debug!("WSLUI API Request: method={}, url={}, body={:?}", method, url, body);
            
            let mut req = match method.to_uppercase().as_str() {
                "POST" => ureq::post(url),
                _ => ureq::get(url),
            };

            req = req.timeout(timeout)
                     .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");

            let resp = if let Some(ref b) = body {
                req.send_json(b)
            } else {
                req.call()
            };

            let resp = resp.map_err(|e| {
                error!("WSLUI API Network Error: {}", e);
                e.to_string()
            })?;

            let date_header = resp.header("Date").map(|s| s.to_string());
            let status = resp.status();
            
            let resp_text = resp.into_string().map_err(|e| {
                error!("WSLUI API Read Body Error: {}", e);
                e.to_string()
            })?;

            debug!("WSLUI API Response: status={}, body={}", status, resp_text);

            let clean_text = resp_text.trim_start_matches('\u{FEFF}').trim();

            let api_resp: ApiResponse<T> = serde_json::from_str(clean_text).map_err(|e| {
                error!("WSLUI API JSON Parse Error: {}", e);
                e.to_string()
            })?;

            if api_resp.err != 0 {
                return Err(format!("API business error: err={}, msg={}", api_resp.err, api_resp.msg));
            }

            Ok((api_resp, date_header))
        };

        match do_request() {
            Ok(res) => Ok(res),
            Err(e) => {
                debug!("WSLUI API Request failed: {}. Retrying after 100ms...", e);
                thread::sleep(Duration::from_millis(200));
                do_request().map_err(|e2| {
                    error!("WSLUI API Retry failed: {}", e2);
                    e2
                })
            }
        }
    }
}
