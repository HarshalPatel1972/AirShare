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

const SERVER_PORT: u16 = 8080;

/// Server state
pub struct ServerState {
    pub shared_dir: PathBuf,
}

impl ServerState {
    pub fn new() -> Self {
        // Use a "shared" folder in the current directory
        let shared_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("shared");

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

pub type SharedServerState = Arc<ServerState>;

/// Start the HTTP file server
pub async fn start_server(state: SharedServerState) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/file/{filename}", get(serve_file))
        .route("/health", get(health_check))
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
