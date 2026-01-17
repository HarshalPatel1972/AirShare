// AirShare - Native Rust Application with Phantom UI

mod discovery;
mod server;

use discovery::{start_beacon, start_listener, DiscoveryState, Peer, SharedDiscoveryState};
use server::{start_server, ServerState, SharedServerState};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
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

/// Tauri command to manually connect to a peer by IP (for hotspot fallback)
#[tauri::command]
async fn manual_connect(
    state: tauri::State<'_, SharedDiscoveryState>,
    ip: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let peer = Peer {
        id: format!("manual-{}", ip),
        ip: ip.clone(),
        name: format!("Device at {}", ip),
        is_holding: false,
        held_file: String::new(),
    };
    
    {
        let mut discovery = state.write().await;
        discovery.peers.insert(peer.id.clone(), peer.clone());
    }
    
    let _ = app_handle.emit("peer-discovered", &peer);
    println!("[Discovery] Manual connect: {}", ip);
    Ok(ip)
}

/// Tauri command to toggle click-through mode
#[tauri::command]
async fn set_click_through(window: tauri::Window, enabled: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())?;
    println!("[Phantom] Click-through: {}", enabled);
    Ok(())
}

/// Tauri command to enter Phantom Mode (transparent overlay)
#[tauri::command]
async fn enter_phantom_mode(window: tauri::Window) -> Result<(), String> {
    println!("[Phantom] Entering Phantom Mode...");
    
    // Set fullscreen
    window.set_fullscreen(true).map_err(|e| e.to_string())?;
    
    // Remove decorations
    window.set_decorations(false).map_err(|e| e.to_string())?;
    
    // Always on top
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    
    // Enable click-through
    window.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
    
    println!("[Phantom] Mode activated!");
    Ok(())
}

/// Tauri command to exit Phantom Mode (back to windowed)
#[tauri::command]
async fn exit_phantom_mode(window: tauri::Window) -> Result<(), String> {
    println!("[Phantom] Exiting Phantom Mode...");
    
    // Disable click-through first
    window.set_ignore_cursor_events(false).map_err(|e| e.to_string())?;
    
    // Exit fullscreen
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    
    // Restore decorations
    window.set_decorations(true).map_err(|e| e.to_string())?;
    
    // Not always on top
    window.set_always_on_top(false).map_err(|e| e.to_string())?;
    
    // Resize to dashboard size
    let _ = window.set_size(tauri::LogicalSize::new(1000.0, 700.0));
    let _ = window.center();
    
    println!("[Phantom] Back to Dashboard mode");
    Ok(())
}

/// Tauri command to simulate a mouse click at current cursor position
#[tauri::command]
fn simulate_click() -> Result<(), String> {
    use enigo::{Enigo, Mouse, Settings, Button};
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.button(Button::Left, enigo::Direction::Click).map_err(|e| e.to_string())?;
    
    println!("[Gesture] Simulated click");
    Ok(())
}

/// Tauri command to simulate scroll
#[tauri::command]
fn simulate_scroll(direction: i32) -> Result<(), String> {
    use enigo::{Enigo, Mouse, Settings, Axis};
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    // Positive = scroll up, Negative = scroll down
    enigo.scroll(direction, Axis::Vertical).map_err(|e| e.to_string())?;
    
    println!("[Gesture] Simulated scroll: {}", direction);
    Ok(())
}

/// Tauri command to simulate media play/pause
#[tauri::command]
fn simulate_media_toggle() -> Result<(), String> {
    use enigo::{Enigo, Keyboard, Settings, Key};
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.key(Key::MediaPlayPause, enigo::Direction::Click).map_err(|e| e.to_string())?;
    
    println!("[Gesture] Simulated Play/Pause");
    Ok(())
}

/// Tauri command to move the real OS cursor to screen coordinates
#[tauri::command]
fn simulate_mouse_move(x: i32, y: i32) -> Result<(), String> {
    use enigo::{Enigo, Mouse, Settings, Coordinate};
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Tauri command to get screen dimensions
#[tauri::command]
fn get_screen_size() -> Result<(i32, i32), String> {
    // Return typical screen size - enigo doesn't have screen size API
    // Frontend will use window.screen.width/height instead
    Ok((1920, 1080))
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let discovery_state: SharedDiscoveryState = Arc::new(RwLock::new(DiscoveryState::new()));
    let server_state: SharedServerState = Arc::new(ServerState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(discovery_state.clone())
        .manage(server_state.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let discovery_for_beacon = discovery_state.clone();
            let discovery_for_listener = discovery_state.clone();

            // === SYSTEM TRAY ===
            let quit_item = MenuItem::with_id(app, "quit", "Quit AirShare", true, None::<&str>)?;
            let status_item = MenuItem::with_id(app, "status", "Status: Scanning...", false, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Toggle Click-Through", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[&status_item, &toggle_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            println!("[Tray] Quit requested");
                            app.exit(0);
                        }
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                // Toggle click-through (this is a simple toggle)
                                let _ = window.set_ignore_cursor_events(true);
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            println!("[Phantom] System tray created");

            // === Enable click-through by default ===
            if let Some(window) = app.get_webview_window("main") {
                // Start with click-through DISABLED so user can interact initially
                // They can enable it via tray or keyboard shortcut
                let _ = window.set_ignore_cursor_events(false);
            }

            // === Background Services ===
            tauri::async_runtime::spawn(async move {
                start_beacon(discovery_for_beacon).await;
            });

            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                start_listener(discovery_for_listener, move |peer: Peer, is_grab_update: bool| {
                    if is_grab_update {
                        let _ = app_handle_clone.emit("grab-update", &peer);
                    } else {
                        let _ = app_handle_clone.emit("peer-discovered", &peer);
                    }
                })
                .await;
            });

            tauri::async_runtime::spawn(async move {
                start_server(server_state).await;
            });

            println!("[AirShare] Phantom UI engine started!");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_grab,
            clear_grab,
            download_file,
            get_device_info,
            manual_connect,
            set_click_through,
            enter_phantom_mode,
            exit_phantom_mode,
            simulate_click,
            simulate_scroll,
            simulate_media_toggle,
            simulate_mouse_move,
            get_screen_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
