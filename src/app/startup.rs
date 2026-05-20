// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use slint::ComponentHandle;
use crate::{AppWindow, AppInfo, AppState};

// Automatically check for updates and expiration at startup
pub fn spawn_check_task(app_handle: slint::Weak<AppWindow>, app_state: Arc<Mutex<AppState>>) {
    let ah = app_handle.clone();
    let app_state_check = app_state.clone();
    
    tokio::spawn(async move {
        let current_v = env!("CARGO_PKG_VERSION");
        // Wait a moment before checking to avoid affecting startup speed
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Read last popup time and check update interval
        let (last_check_time, check_update_days, timezone, is_silent_mode, startup_timestamp) = {
            let state = app_state_check.lock().await;
            let settings = state.config_manager.get_settings();
            let timezone = state.config_manager.get_config().system.timezone.clone();
            (
                settings.check_time.parse::<i64>().unwrap_or(0),
                settings.check_update as i64,
                timezone,
                state.is_silent_mode,
                state.startup_timestamp.load(std::sync::atomic::Ordering::SeqCst)
            )
        };
        
        if is_silent_mode {
            info!("Skipping startup checks (silent mode)");
            return;
        }
        
        let now_ms = chrono::Utc::now().timestamp_millis();
        let interval_ms: i64 = check_update_days * 24 * 60 * 60 * 1000;
        let should_check_update = (now_ms - last_check_time) >= interval_ms;

        info!("Check-update: last={}, now={}, interval={}, should_check_update={}", 
               last_check_time, now_ms, interval_ms, should_check_update);
        
        // If not time to check, skip both expiration and update checks
        if !should_check_update {
            info!("Skipping startup checks (interval not reached)");
            return;
        }

        // Check expiration first
        let expire_time_str = env!("APP_EXPIRE_TIME");
        let expire_time: i64 = expire_time_str.parse().unwrap_or(0);
        
        if expire_time > 0 {
            let mut now = startup_timestamp;
            
            // Wait for background fetch to complete (up to 5 seconds)
            if now <= 0 {
                info!("Waiting for background timestamp synchronization...");
                for _ in 0..50 { // 50 * 100ms = 5s
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    now = app_state_check.lock().await.startup_timestamp.load(std::sync::atomic::Ordering::SeqCst);
                    if now > 0 { 
                        info!("Background timestamp synchronization completed during wait.");
                        break; 
                    }
                }
            }

            // Fallback to synchronous fetch if background fetch failed or timed out
            if now <= 0 {
                warn!("Startup timestamp still not synchronized after waiting, performing fallback sync fetch...");
                // Note: get_standard_time is synchronous, but we are inside tokio::spawn so it only blocks this background task
                now = crate::service::time_service::get_standard_time(&timezone);
            }

            info!("Final timestamp for expiration check: {}", now);

            if now > expire_time {
                let ah_c = ah.clone();
                tokio::spawn(async move {
                    let release = crate::api::common::wslui_latest_release();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = ah_c.upgrade() {
                            app.set_expire_latest_version(release.version.into());
                            app.set_expire_release_date(release.release_date.into());
                            app.set_expire_download_url(release.download_url.into());
                            app.set_show_expire_dialog(true);
                        }
                    });
                });
                
                // Update check-update timestamp
                let mut state = app_state_check.lock().await;
                let _ = state.config_manager.update_check_time();
                info!("App expired! Skipping update check.");
                return;
            }
        }

        // If not expired, then check for updates
        let ah_c = ah.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = ah_c.upgrade() {
                app.global::<AppInfo>().set_checking_update(true);
            }
        });

        match crate::app::updater::check_update(current_v).await {
            Ok(result) => {
                // Update status in AppInfo regardless of popup (used by About page)
                let has_update = result.has_update;
                let latest_version = result.latest_version.clone();
                let release_date = result.release_date.clone();
                let download_url = result.download_url.clone();
                
                let ah_c = ah.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = ah_c.upgrade() {
                        app.global::<AppInfo>().set_checking_update(false);
                        if has_update {
                            app.global::<AppInfo>().set_has_update(true);
                            app.global::<AppInfo>().set_latest_version(latest_version.into());
                            app.global::<AppInfo>().set_latest_release_date(release_date.into());
                            app.global::<AppInfo>().set_update_download_url(download_url.into());
                        }
                    }
                });

                // Only show popup if there's an update (should_check_update is already true here)
                if has_update {
                    let ah_c = ah.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = ah_c.upgrade() {
                            app.set_show_update_dialog(true);
                        }
                    });
                    // Update check-update timestamp
                    let mut state = app_state_check.lock().await;
                    let _ = state.config_manager.update_check_time();
                }
            }
            Err(e) => {
                warn!("Auto check update failed: {}", e);
                let ah_c = ah.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = ah_c.upgrade() {
                        app.global::<AppInfo>().set_checking_update(false);
                    }
                });
            }
        }
    });
}
