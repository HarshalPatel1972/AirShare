// Native Rust HTTP File Server (replaces Go server package)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tower_http::cors::{Any, CorsLayer};

const SERVER_PORT: u16 = 8081; // Changed to 8081 to avoid conflicts

/// Server state
pub struct ServerState {
    pub shared_dir: PathBuf,
}

impl ServerState {
    pub fn new() -> Self {
        // Use the unified AirShare_Downloads folder
        let shared_dir = get_shared_dir();

        // Create the directory if it doesn't exist
        if !shared_dir.exists() {
            let _ = std::fs::create_dir_all(&shared_dir);
        }

        // Create a demo file
        let demo_path = shared_dir.join("demo.txt");
        if !demo_path.exists() {
            let _ = std::fs::write(&demo_path, "Hello from AirShare!\nThis is a demo file.");
            println!("[Server] Created demo.txt in shared folder");
        }

        println!("[Server] Shared directory: {:?}", shared_dir);

        Self { shared_dir }
    }

    pub fn get_shared_dir(&self) -> &PathBuf {
        &self.shared_dir
    }
}

pub fn get_shared_dir() -> std::path::PathBuf {
    dirs::download_dir()
        .unwrap_or(std::path::PathBuf::from("."))
        .join("AirShare_Downloads")
}

pub type SharedServerState = Arc<ServerState>;

/// Start the HTTP file server
pub async fn start_server(state: SharedServerState) {
    println!("[Server] Initializing on port {}...", SERVER_PORT); // Debug log
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/file/{filename}", get(serve_file))
        .route("/health", get(health_check))
        // New Mobile Web Routes
        .route("/mobile", get(handle_mobile_ui))
        .route("/upload", axum::routing::post(handle_upload))
        .route("/files", get(list_files))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", SERVER_PORT);
    println!("[Server] Starting HTTP server on port {}", SERVER_PORT);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[Server] Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[Server] Server error: {}", e);
    }
}

/// Serve a file from the shared directory
async fn serve_file(
    State(state): State<SharedServerState>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let file_path = state.shared_dir.join(&filename);

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, format!("File not found: {}", filename)).into_response();
    }

    match fs::read(&file_path).await {
        Ok(contents) => {
            println!("[Server] Serving file: {}", filename);
            (StatusCode::OK, contents).into_response()
        }
        Err(e) => {
            eprintln!("[Server] Failed to read file {}: {}", filename, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

/// Serve the Mobile Logic HTML
async fn handle_mobile_ui() -> impl IntoResponse {
    let path = PathBuf::from("mobile-web/index.html");
    match fs::read_to_string(&path).await {
        Ok(html) => (StatusCode::OK, axum::response::Html(html)).into_response(),
        Err(_) => {
            // Fallback if running from a different CWD or packaged
             (StatusCode::OK, axum::response::Html(include_str!("../mobile-web/index.html"))).into_response()
        }
    }
}

/// Handle File Upload
async fn handle_upload(
    State(state): State<SharedServerState>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            continue;
        };

        println!("[Server] Receiving upload: {}", file_name);
        
        // Save to AirShare_Downloads
        let file_path = state.shared_dir.join(&file_name);
        
        // Read bytes
        let data = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read upload: {}", e)).into_response(),
        };

        // Write to disk
        if let Err(e) = fs::write(&file_path, &data).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {}", e)).into_response();
        }
        
        println!("[Server] Saved uploaded file: {:?}", file_path);
    }

    (StatusCode::OK, "Upload successful").into_response()
}

/// List files in shared directory
async fn list_files(State(state): State<SharedServerState>) -> impl IntoResponse {
    let mut file_names = Vec::new();

    if let Ok(mut entries) = fs::read_dir(&state.shared_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_file() {
                    if let Ok(name) = entry.file_name().into_string() {
                        file_names.push(name);
                    }
                }
            }
        }
    }

    axum::Json(file_names).into_response()
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "AirShare Server OK"
}

/// Download a file from a URL and save to destination
pub async fn download_file(url: &str, dest_path: &str) -> Result<(), String> {
    println!("[Server] Downloading: {} -> {}", url, dest_path);

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let mut file = fs::File::create(dest_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!("[Server] Download complete: {}", dest_path);
    Ok(())
}
