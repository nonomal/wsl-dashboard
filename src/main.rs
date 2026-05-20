#![windows_subsystem = "windows"]
// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// Import modules
mod wsl;
mod usb;
mod utils;
mod ui;
mod config;
mod app;
mod i18n;
mod network;
mod api;
mod service;

// Re-export types so other modules can continue using crate::AppWindow, crate::AppState etc.
pub use app::state::AppState;
pub use app::runner::*;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    // 1. Handle immediate CLI commands (version/help) before any complex initialization
    if let Some(_) = app::cli::handle_immediate_commands(&args) {
        return;
    }

    #[cfg(all(debug_assertions, windows))]
    {
        // Try to attach to parent console in debug mode so `cargo run` logs are visible.
        // Skip attaching if we are specifically running the scheduler.
        if !args.iter().any(|a| a == "/scheduler") {
            crate::utils::system::attach_console();
        }
    }

    // 2. Bootstrap application environment (Config, i18n, Logging, and Startup Timestamp)
    let ctx = app::launcher::bootstrap(&args).await;
    
    // 3. Handle complex CLI commands that require initialization context (scheduler/clean/initialize)
    if let Some(_) = app::cli::handle_context_commands(&args, &ctx.config_manager, ctx.logging_system.clone()).await {
        return;
    }

    // 4. Silent mode detection
    let is_silent_mode = args.iter().any(|arg| arg.eq_ignore_ascii_case("/silent"));

    // 5. Pre-run maintenance (Single instance check, log cleanup, autostart repair)
    let _instance = match app::lifecycle::pre_run_maintenance(&ctx, is_silent_mode).await {
        Ok(ins) => ins,
        Err(_) => return,
    };

    // 6. Start UI application
    app::runner::run_app(
        ctx.config_manager, 
        ctx.logging_system, 
        is_silent_mode, 
        ctx.startup_ts_atomic
    ).await;
    
    // _instance will be dropped here when main exits
}
