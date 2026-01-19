//! Smart Drop Module
//! 
//! Implements gesture-based copy/paste:
//! - Closed Fist = Ctrl+C (copy selected item)
//! - Open Palm = Ctrl+V (paste)

use enigo::{Enigo, Key, Keyboard, Settings};

/// Simulate Ctrl+C (copy)
/// Called when user makes a closed fist gesture
#[tauri::command]
pub fn simulate_copy() -> Result<String, String> {
    println!("[SmartDrop] 🤜 Simulating Ctrl+C (COPY)");
    
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create enigo: {}", e))?;
    
    // Press Ctrl+C
    enigo.key(Key::Control, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press Ctrl: {}", e))?;
    
    enigo.key(Key::Unicode('c'), enigo::Direction::Click)
        .map_err(|e| format!("Failed to press C: {}", e))?;
    
    enigo.key(Key::Control, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release Ctrl: {}", e))?;
    
    println!("[SmartDrop] ✅ Ctrl+C sent!");
    Ok("Copied!".to_string())
}

/// Simulate Ctrl+V (paste)
/// Called when user opens palm after fist
#[tauri::command]
pub fn simulate_paste() -> Result<String, String> {
    println!("[SmartDrop] 🖐️ Simulating Ctrl+V (PASTE)");
    
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create enigo: {}", e))?;
    
    // Press Ctrl+V
    enigo.key(Key::Control, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press Ctrl: {}", e))?;
    
    enigo.key(Key::Unicode('v'), enigo::Direction::Click)
        .map_err(|e| format!("Failed to press V: {}", e))?;
    
    enigo.key(Key::Control, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release Ctrl: {}", e))?;
    
    println!("[SmartDrop] ✅ Ctrl+V sent!");
    Ok("Pasted!".to_string())
}

/// Get list of files currently in the clipboard
/// Uses PowerShell to read FileDropList
#[tauri::command]
pub fn get_clipboard_files() -> Result<Vec<String>, String> {
    use std::process::Command;
    
    // Check for files in clipboard
    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", "Get-Clipboard -Format FileDropList | Select-Object -ExpandProperty Name"])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
        
    Ok(files)
}

/// Clear the clipboard content
#[tauri::command]
pub fn clear_clipboard() -> Result<(), String> {
    use std::process::Command;
    
    // Use Clear-Clipboard which is more standard, fallback to Set-Clipboard $null
    // We run both to be extremely aggressive about clearing
    let script = "Clear-Clipboard; if ($?) { return }; Set-Clipboard $null";
    
    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    if !output.status.success() {
        return Err("Failed to clear clipboard".to_string());
    }
    
    Ok(())
}
