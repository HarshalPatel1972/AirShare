// AirShare - Native Rust Application (No Go Sidecar)

mod discovery;
mod server;

use discovery::{start_beacon, start_listener, DiscoveryState, Peer, SharedDiscoveryState};
use server::{start_server, ServerState, SharedServerState};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;

/// Tauri command to set grab state
#[tauri::command]
async fn set_grab(
    state: tauri::State<'_, SharedDiscoveryState>,
    filename: String,
) -> Result<(), String> {
    let mut discovery = state.write().await;
    discovery.set_grab(&filename);
    Ok(())
}

/// Tauri command to clear grab state
#[tauri::command]
async fn clear_grab(state: tauri::State<'_, SharedDiscoveryState>) -> Result<(), String> {
    let mut discovery = state.write().await;
    discovery.clear_grab();
    Ok(())
}

/// Tauri command to download a file
#[tauri::command]
async fn download_file(url: String, dest_path: String) -> Result<String, String> {
    server::download_file(&url, &dest_path).await?;
    Ok(dest_path)
}

/// Tauri command to get local device info
#[tauri::command]
async fn get_device_info(
    state: tauri::State<'_, SharedDiscoveryState>,
) -> Result<serde_json::Value, String> {
    let discovery = state.read().await;
    Ok(serde_json::json!({
        "id": discovery.device_id,
        "name": discovery.device_name,
        "ip": discovery.local_ip
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize discovery state
    let discovery_state: SharedDiscoveryState = Arc::new(RwLock::new(DiscoveryState::new()));

    // Initialize server state
    let server_state: SharedServerState = Arc::new(ServerState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(discovery_state.clone())
        .manage(server_state.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let discovery_for_beacon = discovery_state.clone();
            let discovery_for_listener = discovery_state.clone();

            // Start the UDP beacon broadcaster
            tauri::async_runtime::spawn(async move {
                start_beacon(discovery_for_beacon).await;
            });

            // Start the UDP listener
            tauri::async_runtime::spawn(async move {
                start_listener(discovery_for_listener, move |peer: Peer, is_grab_update: bool| {
                    if is_grab_update {
                        // Emit grab-update event
                        let _ = app_handle.emit("grab-update", &peer);
                    } else {
                        // Emit peer-discovered event
                        let _ = app_handle.emit("peer-discovered", &peer);
                    }
                })
                .await;
            });

            // Start the HTTP file server
            tauri::async_runtime::spawn(async move {
                start_server(server_state).await;
            });

            println!("[AirShare] Native Rust engine started!");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_grab,
            clear_grab,
            download_file,
            get_device_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
