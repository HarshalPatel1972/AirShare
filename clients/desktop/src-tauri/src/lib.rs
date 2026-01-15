// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

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
                            println!("[Go Engine] {}", output);
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

