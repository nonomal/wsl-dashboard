// SPDX-FileCopyrightText: Copyright (c) 2026 owu <wqh@live.com>
// SPDX-License-Identifier: GPL-3.0-only

use tracing::{info, error};
use crate::config;
use crate::network;
use crate::app::WSL_INIT_SCRIPT;

// Execute network sync and USB auto-attach tasks triggered by Task Scheduler (/scheduler)
pub async fn run_scheduler_task(args: &[String], pos: usize, config_manager: &config::ConfigManager) {
    crate::utils::system::attach_console();

    info!(">>> [START] Network sync command detected via /scheduler <<<");

    // 4.0 Cleanup legacy startup scripts (vbs) asynchronously with timeout protection
    crate::utils::system::cleanup_legacy_vbs_startup();
    
    // 4.1 Load all necessary configurations
    let instances_path = config::ConfigManager::get_instances_path();
    let container = config::instances::load_instances(&instances_path);
    let net_config = config_manager.get_network_config();
    let rules = &net_config.port_proxies;
    
    // 4.2 Identify target distros for synchronization and auto-start
    let mut target_distros: std::collections::HashSet<String> = std::collections::HashSet::new();
    if pos + 1 < args.len() && !args[pos + 1].starts_with('/') {
        target_distros.insert(args[pos + 1].clone());
    } else {
        // Source 1: Distros with port proxy rules
        for r in rules {
            target_distros.insert(r.distro_name.clone());
        }

        // Source 2: Distros with auto-startup enabled
        for (name, inst) in &container.instances {
            if inst.auto_startup {
                target_distros.insert(name.clone());
            }
        }

        // Source 3: Distros with USB auto-attach configurations
        let usb_config = config_manager.get_usb_config();
        for device in &usb_config.auto_attach_list {
            target_distros.insert(device.distribution.clone());
        }
    }

    // 4.3 Background startup for target distros (if configured for auto-startup)
    let mut distros_spawned = 0;
    for name in &target_distros {
        if let Some(inst_config) = container.instances.get(name) {
            if inst_config.auto_startup {
                info!("Distro '{}' marked for auto-start. Spawning background init script...", name);
                #[cfg(windows)]
                {
                    use std::process::Command;
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    
                    let _ = Command::new("wsl")
                        .args(&["-d", name, "-u", "root", WSL_INIT_SCRIPT, "start"])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn();
                    distros_spawned += 1;
                }
            }
        }
    }
    
    // 4.4 Wait for network interfaces to stabilize if any distros were spawned
    if distros_spawned > 0 {
        info!("Waiting 5 seconds for {} spawned distros to stabilize network interfaces...", distros_spawned);
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    } else {
        info!("No new distros were spawned. Proceeding with existing running instances.");
    }
    
    // 4.5 Execute synchronization for each target distro
    for name in target_distros {
        let distro_rules: Vec<_> = rules.iter().filter(|r| r.distro_name == name).cloned().collect();
        if distro_rules.is_empty() {
            info!("No rules found for distro '{}', skipping.", name);
            continue;
        }
        
        info!(">>> Executing elevated sync for '{}' (Total Rules: {})", name, distro_rules.len());
        // sync_port_proxies has internal 10-retry logic for IP fetching
        if let Err(e) = network::port_proxy::sync_port_proxies(&name, &distro_rules) {
            error!("Sync FAILED for '{}': {}", name, e);
        } else {
            info!("Sync SUCCESS for '{}'.", name);
        }
    }
    
    // 4.6 Auto-attach USB devices if configured
    info!(">>> [START] USB auto-attach synchronization <<<");
    let usb_config = config_manager.get_usb_config().clone();
    
    if !usb_config.auto_attach_list.is_empty() {
         // Check if any WSL 2 instance is running (required for usbipd attach)
         let is_any_running = {
            let mut cmd = std::process::Command::new("wsl");
            cmd.args(["-l", "-v"]);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
            cmd.env("WSL_UTF8", "1");
            
            match cmd.output() {
                Ok(out) => {
                    let stdout = crate::wsl::decoder::decode_output(&out.stdout);
                    stdout.lines()
                        .skip(1) // Skip header
                        .any(|line| {
                            let lower = line.to_lowercase();
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            // Must be Running AND Version 2
                            lower.contains("running") && parts.iter().any(|&p| p == "2")
                        })
                }
                Err(_) => false,
            }
        };
        
        if is_any_running {
            info!("Found running WSL 2 instance. Processing {} auto-attach device(s)...", usb_config.auto_attach_list.len());
            for device in &usb_config.auto_attach_list {
                info!("Auto-attaching USB device: BusId={}, VidPid={}, TargetDistro='{}'", 
                    device.bus_id, device.vid_pid, device.distribution);
                
                // UsbManager::attach internally performs 'usbipd bind' then 'usbipd attach'
                match crate::usb::UsbManager::attach(&device.bus_id, &device.distribution).await {
                    Ok(_) => info!("SUCCESS: USB device {} attached.", device.bus_id),
                    Err(e) => {
                         error!("FAILED to auto-attach USB device {}: {}", device.bus_id, e);
                    }
                }
            }
        } else {
            info!("No running WSL 2 instance found. Skipping USB auto-attach.");
        }
    } else {
        info!("No USB devices configured for auto-attach.");
    }
    info!(">>> [FINISH] USB auto-attach synchronization completed. <<<");
    
    info!(">>> [FINISH] All scheduled network tasks completed. <<<");
}
