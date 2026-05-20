// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::api::common::wslui_standard_time;
use crate::utils::time::internet_standard_time;

// Aggregate to get standard timestamp (milliseconds)
// Priority: WSLUI API, fallback to generic standard_time if failed
pub fn get_standard_time(timezone: &str) -> i64 {
    let ts = wslui_standard_time();
    if ts > 0 {
        return ts;
    }
    
    internet_standard_time(timezone)
}
