package main

import (
	"fmt"
	"time"
)

func main() {
	// Initial greeting to stdout - will be captured by Tauri sidecar
	fmt.Println("Hello from AirShare Go Engine!")

	// Keep the sidecar running - it will be managed by Tauri
	for {
		time.Sleep(time.Hour)
	}
}
