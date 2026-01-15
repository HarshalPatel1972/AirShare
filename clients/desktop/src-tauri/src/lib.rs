// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;
use serde::{Deserialize, Serialize};

/// Peer data received from Go sidecar (updated for grab state)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Peer {
    id: String,
    ip: String,
    name: String,
    #[serde(rename = "isHolding", default)]
    is_holding: bool,
    #[serde(rename = "heldFile", default)]
    held_file: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn send_to_sidecar(command: String) -> Result<String, String> {
    // For now, just log the command - the grab state is already broadcast via UDP
    // In a future version, we could use stdin to communicate complex commands
    println!("[Tauri] Command: {}", command);
    Ok(format!("Logged: {}", command))
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
                                let json_str = output_str.trim_start_matches("[PEER_FOUND]").trim();
                                
                                match serde_json::from_str::<Peer>(json_str) {
                                    Ok(peer) => {
                                        println!("[Discovery] Found peer: {} at {}", peer.name, peer.ip);
                                        if let Err(e) = app_handle.emit("peer-discovered", &peer) {
                                            eprintln!("[Error] Failed to emit peer-discovered: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[Error] Failed to parse peer JSON: {} - {}", e, json_str);
                                    }
                                }
                            } 
                            // Check if this is a grab update event
                            else if output_str.starts_with("[GRAB_UPDATE]") {
                                let json_str = output_str.trim_start_matches("[GRAB_UPDATE]").trim();
                                
                                match serde_json::from_str::<Peer>(json_str) {
                                    Ok(peer) => {
                                        println!("[Transfer] Grab update from {}: holding={}, file={}", 
                                            peer.name, peer.is_holding, peer.held_file);
                                        if let Err(e) = app_handle.emit("grab-update", &peer) {
                                            eprintln!("[Error] Failed to emit grab-update: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[Error] Failed to parse grab JSON: {} - {}", e, json_str);
                                    }
                                }
                            }
                            // Check for download complete
                            else if output_str.starts_with("[DOWNLOAD_COMPLETE]") {
                                let path = output_str.trim_start_matches("[DOWNLOAD_COMPLETE]").trim();
                                println!("[Transfer] Download complete: {}", path);
                                let _ = app_handle.emit("download-complete", path);
                            }
                            else {
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
        .invoke_handler(tauri::generate_handler![greet, send_to_sidecar])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
