package discovery

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"sync"
	"time"

	"github.com/google/uuid"
)

const (
	DiscoveryPort = 9988
	BroadcastAddr = "255.255.255.255:9988"
	ServicePort   = 8080
	BeaconInterval = 1 * time.Second
)

// BeaconPacket is the payload broadcasted by each device
type BeaconPacket struct {
	DeviceID    string `json:"deviceId"`
	DeviceName  string `json:"deviceName"`
	ServicePort int    `json:"servicePort"`
}

// Peer represents a discovered peer
type Peer struct {
	ID   string `json:"id"`
	IP   string `json:"ip"`
	Name string `json:"name"`
}

// Discovery manages the UDP discovery protocol
type Discovery struct {
	deviceID   string
	deviceName string
	peers      map[string]*Peer
	peersMu    sync.RWMutex
	stopChan   chan struct{}
}

// New creates a new Discovery instance
func New() *Discovery {
	hostname, err := os.Hostname()
	if err != nil {
		hostname = "Unknown"
	}

	return &Discovery{
		deviceID:   uuid.New().String(),
		deviceName: hostname,
		peers:      make(map[string]*Peer),
		stopChan:   make(chan struct{}),
	}
}

// Start begins broadcasting and listening for peers
func (d *Discovery) Start() error {
	// Start the beacon broadcaster
	go d.startBeacon()

	// Start the listener
	go d.startListener()

	return nil
}

// Stop halts the discovery service
func (d *Discovery) Stop() {
	close(d.stopChan)
}

// startBeacon broadcasts our presence every second
func (d *Discovery) startBeacon() {
	// For Windows compatibility, we use DialUDP with broadcast address
	broadcastAddr, err := net.ResolveUDPAddr("udp4", BroadcastAddr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "[Discovery] Failed to resolve broadcast address: %v\n", err)
		return
	}

	// Bind to any local address
	localAddr := &net.UDPAddr{IP: net.IPv4zero, Port: 0}
	conn, err := net.DialUDP("udp4", localAddr, broadcastAddr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "[Discovery] Failed to create broadcast socket: %v\n", err)
		return
	}
	defer conn.Close()

	packet := BeaconPacket{
		DeviceID:    d.deviceID,
		DeviceName:  d.deviceName,
		ServicePort: ServicePort,
	}

	data, err := json.Marshal(packet)
	if err != nil {
		fmt.Fprintf(os.Stderr, "[Discovery] Failed to marshal beacon: %v\n", err)
		return
	}

	fmt.Println("[Discovery] Beacon started, broadcasting every 1s...")

	ticker := time.NewTicker(BeaconInterval)
	defer ticker.Stop()

	for {
		select {
		case <-d.stopChan:
			return
		case <-ticker.C:
			_, err := conn.Write(data)
			if err != nil {
				fmt.Fprintf(os.Stderr, "[Discovery] Beacon send error: %v\n", err)
			}
		}
	}
}

// startListener listens for beacon packets from other devices
func (d *Discovery) startListener() {
	// Use udp4 explicitly for Windows compatibility
	addr := &net.UDPAddr{
		Port: DiscoveryPort,
		IP:   net.IPv4zero,
	}

	conn, err := net.ListenUDP("udp4", addr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "[Discovery] Failed to start listener on port %d: %v\n", DiscoveryPort, err)
		fmt.Fprintf(os.Stderr, "[Discovery] This may be due to firewall or another process using the port.\n")
		return
	}
	defer conn.Close()

	fmt.Printf("[Discovery] Listener started on port %d\n", DiscoveryPort)

	buffer := make([]byte, 1024)

	for {
		select {
		case <-d.stopChan:
			return
		default:
			conn.SetReadDeadline(time.Now().Add(2 * time.Second))
			n, remoteAddr, err := conn.ReadFromUDP(buffer)
			if err != nil {
				if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
					continue
				}
				continue
			}

			var packet BeaconPacket
			if err := json.Unmarshal(buffer[:n], &packet); err != nil {
				continue
			}

			// Ignore our own broadcasts
			if packet.DeviceID == d.deviceID {
				continue
			}

			// Check if this is a new peer
			d.peersMu.Lock()
			if _, exists := d.peers[packet.DeviceID]; !exists {
				peer := &Peer{
					ID:   packet.DeviceID,
					IP:   remoteAddr.IP.String(),
					Name: packet.DeviceName,
				}
				d.peers[packet.DeviceID] = peer

				// Output to stdout for Rust to capture
				peerJSON, _ := json.Marshal(peer)
				fmt.Printf("[PEER_FOUND] %s\n", string(peerJSON))
			}
			d.peersMu.Unlock()
		}
	}
}

// GetDeviceID returns this device's unique ID
func (d *Discovery) GetDeviceID() string {
	return d.deviceID
}

// GetDeviceName returns this device's name
func (d *Discovery) GetDeviceName() string {
	return d.deviceName
}
