// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::Path;
use crate::{AppWindow, AppState, i18n};
use crate::wsl::ops::compress;

pub fn setup(app: &AppWindow, app_handle: slint::Weak<AppWindow>, app_state: Arc<Mutex<AppState>>) {
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    
    // 1. Open Dialog Callback
    app.on_open_compress_dialog(move |distro_name| {
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let name = distro_name.to_string();
        
        tokio::spawn(async move {
            let executor = {
                let state = as_ptr.lock().await;
                state.wsl_dashboard.executor().clone()
            };
            
            // Get VHDX info
            let info_res = crate::wsl::ops::info::get_distro_information(&executor, &name).await;
            
            let mut vhdx_size = "---".to_string();
            let mut free_space = "---".to_string();
            let mut sufficient = false;
            let mut backup_path = "".to_string();
            let mut script_url = "".to_string();
            let mut is_wsl2 = true;
            let mut is_sparse = false;

            if info_res.success {
                if let Some(ref info) = info_res.data {
                    vhdx_size = if !info.vhdx_size.is_empty() {
                        info.vhdx_size.clone()
                    } else {
                        info.actual_used.clone()
                    };
                    
                    is_wsl2 = info.wsl_version == "WSL2";
                    is_sparse = info.is_sparse;

                    // Determine backup path and space check logic
                    let base_path = if !info.vhdx_path.is_empty() {
                        Path::new(&info.vhdx_path).to_path_buf()
                    } else {
                        Path::new(&info.install_location).to_path_buf()
                    };

                    if base_path.as_os_str().len() >= 3 {
                        let path_str = base_path.to_string_lossy();
                        let drive = &path_str[..3];
                        let free_bytes = crate::utils::system::get_disk_free_space(drive);
                        free_space = format!("{:.2} GB", free_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
                        
                        let vhdx_bytes = if vhdx_size.contains("GB") {
                            (vhdx_size.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0 * 1024.0) as u64
                        } else if vhdx_size.contains("MB") {
                            (vhdx_size.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0) as u64
                        } else {
                            0
                        };
                        sufficient = free_bytes > (vhdx_bytes + 2 * 1024 * 1024 * 1024);
                    }
                    
                    backup_path = if base_path.is_dir() {
                        base_path.join(format!("{}.tar", name)).to_string_lossy().to_string()
                    } else {
                        format!("{}.tar", base_path.display())
                    };

                    let distro_helper = crate::api::common::wslui_helper_distro();
                    script_url = distro_helper.compress_script.url;
                }
            }
            
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = ah.upgrade() {
                    app.set_compress_distro_name(name.into());
                    app.set_compress_vhdx_size(vhdx_size.into());
                    app.set_compress_free_space(free_space.into());
                    app.set_compress_space_sufficient(sufficient);
                    app.set_compress_backup_path(backup_path.into());
                    app.set_compress_script_url(script_url.into());
                    app.set_compress_is_wsl2(is_wsl2);
                    app.set_compress_is_sparse(is_sparse);
                    app.set_compress_enable_sparse(is_sparse); // Default to true if already sparse
                    app.set_show_compress_dialog(true);
                }
            });
        });
    });

    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    
    // 2. Confirm Compression Callback
    app.on_confirm_compress(move |distro_name, cleanup, backup, enable_sparse, script_url| {
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let name = distro_name.to_string();
        let url = script_url.to_string();
        
        let name_for_ui = name.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = ah.upgrade() {
                app.set_show_compress_dialog(false);
                app.set_task_status_visible(true);
                let msg = i18n::tr("task.compress_starting", &[name_for_ui.clone()]);
                app.set_task_status_text(msg.into());
            }
        });
        
        let ah_task = app_handle.clone();
        tokio::spawn(async move {
            let _guard = crate::ui::data::BusyGuard::new();
            let executor = {
                let state = as_ptr.lock().await;
                state.wsl_dashboard.mark_distro_stopped(&name).await;
                state.wsl_dashboard.executor().clone()
            };
            
            let ah_prog = ah_task.clone();
            let name_prog = name.clone();
            let progress_callback = move |key: &str| {
                let ah_inner = ah_prog.clone();
                let name_inner = name_prog.clone();
                let key_inner = key.to_string();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = ah_inner.upgrade() {
                        let msg = i18n::tr(&key_inner, &[name_inner]);
                        app.set_task_status_text(msg.into());
                    }
                });
            };

            let result = compress::compress_vhdx(&executor, &name, cleanup, backup, enable_sparse, &url, progress_callback).await;

            
            let name_inner = name.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = ah_task.upgrade() {
                    if result.success {
                        let saved_size = result.output.clone();
                        let msg = i18n::tr("task.compress_success", &[name_inner, saved_size]);
                        app.set_task_status_text(msg.into());
                        // Refresh distro list using the centralized UI refresh function
                        let ah_refresh = ah_task.clone();
                        let as_refresh = as_ptr.clone();
                        tokio::spawn(async move {
                            // First, ensure the dashboard cache is updated
                            let dashboard = {
                                let state = as_refresh.lock().await;
                                state.wsl_dashboard.clone()
                            };
                            let _ = dashboard.refresh_distros().await;
                            
                            // Then update UI
                            crate::ui::data::refresh_distros_ui(ah_refresh, as_refresh).await;
                        });
                    } else {
                        let err_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
                        let msg = i18n::tr("task.compress_failed", &[name_inner, err_msg]);
                        app.set_task_status_text(msg.into());
                    }
                }
            });
        });
    });
}
