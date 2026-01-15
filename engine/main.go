package main

import (
	"bufio"
	"fmt"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"

	"airshare-engine/discovery"
	"airshare-engine/server"
)

var disc *discovery.Discovery
var srv *server.Server

func main() {
	// Initial greeting to stdout - will be captured by Tauri sidecar
	fmt.Println("Hello from AirShare Go Engine!")

	// Start the discovery service
	disc = discovery.New()
	fmt.Printf("[INFO] Device ID: %s\n", disc.GetDeviceID())
	fmt.Printf("[INFO] Device Name: %s\n", disc.GetDeviceName())
	fmt.Printf("[INFO] Local IP: %s\n", discovery.GetLocalIP())
	fmt.Println("[INFO] Starting discovery service...")

	if err := disc.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "[ERROR] Failed to start discovery: %v\n", err)
		os.Exit(1)
	}

	// Start HTTP file server
	srv = server.New()
	if err := srv.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "[ERROR] Failed to start server: %v\n", err)
	}

	fmt.Println("[INFO] AirShare engine running. Waiting for commands...")

	// Start stdin command listener in goroutine
	go listenForCommands()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	fmt.Println("[INFO] Shutting down...")
	disc.Stop()
}

// listenForCommands reads stdin for commands from Tauri
func listenForCommands() {
	scanner := bufio.NewScanner(os.Stdin)
	
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		parts := strings.SplitN(line, " ", 2)
		cmd := parts[0]

		switch cmd {
		case "GRAB":
			if len(parts) > 1 {
				filename := parts[1]
				disc.SetGrab(filename)
				fmt.Printf("[CMD] Grab started: %s\n", filename)
			}

		case "RELEASE":
			disc.ClearGrab()
			fmt.Println("[CMD] Grab released")

		case "DOWNLOAD":
			if len(parts) > 1 {
				// Format: DOWNLOAD http://ip:port/file/name dest_path
				args := strings.SplitN(parts[1], " ", 2)
				if len(args) == 2 {
					url := args[0]
					destPath := args[1]
					go func() {
						if err := srv.DownloadFile(url, destPath); err != nil {
							fmt.Fprintf(os.Stderr, "[ERROR] Download failed: %v\n", err)
						} else {
							fmt.Printf("[DOWNLOAD_COMPLETE] %s\n", destPath)
						}
					}()
				}
			}

		case "GET_IP":
			fmt.Printf("[LOCAL_IP] %s\n", discovery.GetLocalIP())

		case "LIST_FILES":
			// List files in shared directory
			files, err := filepath.Glob(filepath.Join(srv.GetSharedDir(), "*"))
			if err == nil {
				for _, f := range files {
					fmt.Printf("[FILE] %s\n", filepath.Base(f))
				}
			}

		default:
			fmt.Fprintf(os.Stderr, "[WARN] Unknown command: %s\n", cmd)
		}
	}
}

