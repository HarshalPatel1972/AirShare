package main

import (
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"airshare-engine/discovery"
)

func main() {
	// Initial greeting to stdout - will be captured by Tauri sidecar
	fmt.Println("Hello from AirShare Go Engine!")

	// Start the discovery service
	disc := discovery.New()
	fmt.Printf("[INFO] Device ID: %s\n", disc.GetDeviceID())
	fmt.Printf("[INFO] Device Name: %s\n", disc.GetDeviceName())
	fmt.Println("[INFO] Starting discovery service...")

	if err := disc.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "[ERROR] Failed to start discovery: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("[INFO] Discovery service running. Listening for peers...")

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	fmt.Println("[INFO] Shutting down...")
	disc.Stop()
}
