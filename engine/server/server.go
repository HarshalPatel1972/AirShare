package server

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
)

const (
	ServerPort = 8080
	SharedDir  = "./shared" // Directory for shared files
)

// Server handles HTTP file serving and downloading
type Server struct {
	sharedDir string
	port      int
}

// New creates a new file server
func New() *Server {
	return &Server{
		sharedDir: SharedDir,
		port:      ServerPort,
	}
}

// Start begins the HTTP file server
func (s *Server) Start() error {
	// Create shared directory if it doesn't exist
	if err := os.MkdirAll(s.sharedDir, 0755); err != nil {
		fmt.Fprintf(os.Stderr, "[Server] Failed to create shared dir: %v\n", err)
	}

	// Create a demo file for testing
	s.createDemoFile()

	mux := http.NewServeMux()
	
	// File serving endpoint
	mux.HandleFunc("/file/", s.handleFileServe)
	
	// Health check
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	})

	fmt.Printf("[Server] Starting HTTP server on port %d\n", s.port)
	
	go func() {
		if err := http.ListenAndServe(fmt.Sprintf(":%d", s.port), mux); err != nil {
			fmt.Fprintf(os.Stderr, "[Server] HTTP server error: %v\n", err)
		}
	}()

	return nil
}

// handleFileServe serves files from the shared directory
func (s *Server) handleFileServe(w http.ResponseWriter, r *http.Request) {
	// Extract filename from URL: /file/filename.ext
	filename := filepath.Base(r.URL.Path)
	
	if filename == "" || filename == "." {
		http.Error(w, "Invalid filename", http.StatusBadRequest)
		return
	}

	filePath := filepath.Join(s.sharedDir, filename)
	
	// Check if file exists
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		http.Error(w, "File not found", http.StatusNotFound)
		return
	}

	fmt.Printf("[Server] Serving file: %s\n", filename)
	
	// Set content disposition for download
	w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%s", filename))
	
	http.ServeFile(w, r, filePath)
}

// DownloadFile downloads a file from a remote URL
func (s *Server) DownloadFile(url string, destPath string) error {
	fmt.Printf("[Server] Downloading from %s to %s\n", url, destPath)

	resp, err := http.Get(url)
	if err != nil {
		return fmt.Errorf("failed to download: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	// Create destination file
	out, err := os.Create(destPath)
	if err != nil {
		return fmt.Errorf("failed to create file: %v", err)
	}
	defer out.Close()

	// Copy data
	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return fmt.Errorf("failed to write file: %v", err)
	}

	fmt.Printf("[Server] Downloaded successfully: %s\n", destPath)
	return nil
}

// createDemoFile creates a test file for demo purposes
func (s *Server) createDemoFile() {
	demoPath := filepath.Join(s.sharedDir, "demo.txt")
	if _, err := os.Stat(demoPath); os.IsNotExist(err) {
		content := []byte("Hello from AirShare! This is a demo file for testing P2P transfer.")
		if err := os.WriteFile(demoPath, content, 0644); err != nil {
			fmt.Fprintf(os.Stderr, "[Server] Failed to create demo file: %v\n", err)
		} else {
			fmt.Println("[Server] Created demo.txt in shared folder")
		}
	}
}

// GetSharedDir returns the shared directory path
func (s *Server) GetSharedDir() string {
	return s.sharedDir
}
