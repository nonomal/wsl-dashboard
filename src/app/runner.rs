// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
use slint::{ComponentHandle, Model};

// Include Slint modules locally in this runner
slint::include_modules!();

use crate::app::{AppState, APP_NAME, APP_ID, PROJECT_REPOSITORY, GITHUB_ISSUES};
use crate::config::ConfigManager;
use crate::utils::logging::LoggingSystem;
use crate::ui;
use crate::i18n;

// Run the main GUI application
pub async fn run_app(config_manager: ConfigManager, logging_system: LoggingSystem, is_silent_mode: bool, startup_timestamp: std::sync::Arc<std::sync::atomic::AtomicI64>) {
    let settings = config_manager.get_settings().clone();
    let tray_settings = config_manager.get_tray_settings().clone();
    let system_language = config_manager.get_config().system.system_language.clone();

    // 1. Create app state
    // Note: We manually unwrap the atomic value once as initial value, or refactor AppState::new
    let initial_ts = startup_timestamp.load(std::sync::atomic::Ordering::SeqCst);
    let app_state = Arc::new(Mutex::new(AppState::new(config_manager, logging_system, is_silent_mode, initial_ts)));
    
    // Store the passed atomic reference into AppState (since AppState::new creates a new atomic internally, we need to sync or store the reference directly)
    // For simplicity, we modify AppState to share this atomic, or do a sync here
    {
        let state_clone = app_state.clone();
        tokio::spawn(async move {
            // Loop until atomic value is not 0 (or use a more elegant notification mechanism)
            // This ensures the atomic inside AppState eventually syncs with the result from main
            loop {
                let ts = startup_timestamp.load(std::sync::atomic::Ordering::SeqCst);
                if ts > 0 {
                    let lock = state_clone.lock().await;
                    lock.startup_timestamp.store(ts, std::sync::atomic::Ordering::SeqCst);
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }
    
    // 2. Create Slint window
    let app = AppWindow::new().expect("Failed to create app");
    app.set_system_language(system_language.into());
    
    // 3. Register i18n callback
    app.global::<AppI18n>().on_t(|key, args| {
        let args_vec: Vec<String> = args.iter().map(|s: slint::SharedString| s.to_string()).collect();
        i18n::tr(&key, &args_vec).into()
    });

    // Initialize localization language code
    let current_lang = i18n::current_lang();
    app.global::<AppI18n>().set_locale_code(current_lang.into());

    // Trigger initial evaluation of all i18n properties
    app.global::<AppI18n>().set_version(1);
    
    // Set version and URL
    app.global::<AppInfo>().set_version(env!("CARGO_PKG_VERSION").into());
    app.global::<AppInfo>().set_project_repository(PROJECT_REPOSITORY.into());
    app.global::<AppInfo>().set_issues_url(format!("{}{}", PROJECT_REPOSITORY, GITHUB_ISSUES).into());

    // 4. Initialize system tray
    if let Err(e) = crate::app::tray::SystemTray::initialize(app.as_weak(), !is_silent_mode) {
        error!("Failed to initialize system tray: {}", e);
    }

    app.on_reinit_tray({
        let ah = app.as_weak();
        move || {
            let current_visible = if let Some(app) = ah.upgrade() {
                app.get_is_window_visible()
            } else {
                false
            };
            info!("Re-initializing tray, current visibility: {}", current_visible);
            if let Err(e) = crate::app::tray::SystemTray::initialize(ah.clone(), current_visible) {
                error!("Failed to re-initialize system tray: {}", e);
            }
        }
    });

    // 5. Load settings to UI (critical for i18n, font and theme)
    ui::data::load_settings_to_ui(&app, &app_state, &settings, &tray_settings).await;

    // 6. Initialize theme watcher if system theme sync is enabled
    if settings.system_color {
        match crate::utils::theme::ThemeWatcher::new(app.as_weak()) {
            Ok(watcher) => {
                let theme = crate::utils::theme::ThemeWatcher::get_current_theme();
                app.global::<Theme>().set_dark_mode(theme == crate::utils::theme::Theme::Dark);
                
                let mut state = app_state.lock().await;
                state.theme_watcher = Some(watcher);
            }
            Err(e) => {
                error!("Failed to initialize ThemeWatcher: {}", e);
            }
        }
    }

    // 7. Set up UI handlers
    ui::handlers::setup(&app, app.as_weak(), app_state.clone()).await;
    
    // 8. Refresh initial data (distro list)
    ui::data::refresh_data(app.as_weak(), app_state.clone()).await;

    // 9. Start background tasks (update check, WSL/USB status monitoring)
    crate::app::startup::spawn_check_task(app.as_weak(), app_state.clone());
    crate::app::tasks::spawn_wsl_monitor(app.as_weak(), app_state.clone());
    crate::app::tasks::spawn_usb_monitor(app.as_weak());
    crate::app::tasks::spawn_state_listener(app.as_weak(), app_state.clone());
    crate::app::tasks::spawn_wakeup_listener(app.as_weak());

    // 10. Show window and center it
    crate::app::window::show_and_center(&app, is_silent_mode);
    
    // 11. Run application event loop with keep-alive timer to prevent exit when hidden
    let keep_alive_timer = slint::Timer::default();
    keep_alive_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(1), || {
        // Keep-alive heartbeat
    });

    info!("Starting {} (ID: {})...", APP_NAME, APP_ID);
    slint::run_event_loop().expect("Failed to run Slint event loop");

    // 12. Handle cleanup on exit
    crate::app::tasks::handle_app_exit(&app, &app_state).await;
}
