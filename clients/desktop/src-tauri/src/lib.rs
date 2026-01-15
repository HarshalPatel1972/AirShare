// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;
use serde::{Deserialize, Serialize};

/// Peer data received from Go sidecar
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Peer {
    id: String,
    ip: String,
    name: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Get app handle for emitting events
            let app_handle = app.handle().clone();

            // Spawn the Go sidecar on app launch
            let shell = app.shell();
            let sidecar_command = shell.sidecar("airshare-engine")
                .expect("Failed to create sidecar command");
            
            let (mut rx, _child) = sidecar_command
                .spawn()
                .expect("Failed to spawn sidecar");

            // Spawn a task to capture stdout from the Go sidecar
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_shell::process::CommandEvent;
                
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => {
                            let output = String::from_utf8_lossy(&line);
                            let output_str = output.trim();
                            
                            // Check if this is a peer discovery event
                            if output_str.starts_with("[PEER_FOUND]") {
                                // Extract the JSON part
                                let json_str = output_str.trim_start_matches("[PEER_FOUND]").trim();
                                
                                // Parse the peer JSON
                                match serde_json::from_str::<Peer>(json_str) {
                                    Ok(peer) => {
                                        println!("[Discovery] Found peer: {} at {}", peer.name, peer.ip);
                                        
                                        // Emit the event to the frontend
                                        if let Err(e) = app_handle.emit("peer-discovered", &peer) {
                                            eprintln!("[Error] Failed to emit peer-discovered: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[Error] Failed to parse peer JSON: {} - {}", e, json_str);
                                    }
                                }
                            } else {
                                println!("[Go Engine] {}", output_str);
                            }
                        }
                        CommandEvent::Stderr(line) => {
                            let output = String::from_utf8_lossy(&line);
                            eprintln!("[Go Engine Error] {}", output);
                        }
                        CommandEvent::Error(error) => {
                            eprintln!("[Sidecar Error] {}", error);
                        }
                        CommandEvent::Terminated(status) => {
                            println!("[Go Engine] Terminated with status: {:?}", status);
                            break;
                        }
                        _ => {}
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

