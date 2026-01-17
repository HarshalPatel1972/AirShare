// Native Rust UDP Discovery (replaces Go discovery package)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

const DISCOVERY_PORT: u16 = 9988;
const BEACON_INTERVAL_MS: u64 = 1000;
const BROADCAST_ADDR: &str = "255.255.255.255:9988";
// Multicast address for better hotspot compatibility
const MULTICAST_ADDR: &str = "224.0.0.251:9988";

/// Beacon packet broadcast over UDP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconPacket {
    pub id: String,
    pub ip: String,
    pub name: String,
    #[serde(rename = "isHolding", default)]
    pub is_holding: bool,
    #[serde(rename = "heldFile", default)]
    pub held_file: String,
}

/// Discovered peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub name: String,
    #[serde(rename = "isHolding")]
    pub is_holding: bool,
    #[serde(rename = "heldFile")]
    pub held_file: String,
}

/// Discovery state shared across async tasks
pub struct DiscoveryState {
    pub device_id: String,
    pub device_name: String,
    pub local_ip: String,
    pub is_holding: bool,
    pub held_file: String,
    pub peers: HashMap<String, Peer>,
}

impl DiscoveryState {
    pub fn new() -> Self {
        let device_id = uuid::Uuid::new_v4().to_string();
        let device_name = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        println!("[Discovery] Device ID: {}", device_id);
        println!("[Discovery] Device Name: {}", device_name);
        println!("[Discovery] Local IP: {}", local_ip);

        Self {
            device_id,
            device_name,
            local_ip,
            is_holding: false,
            held_file: String::new(),
            peers: HashMap::new(),
        }
    }

    pub fn set_grab(&mut self, filename: &str) {
        self.is_holding = true;
        self.held_file = filename.to_string();
        println!("[Discovery] Grab: {}", filename);
    }

    pub fn clear_grab(&mut self) {
        self.is_holding = false;
        self.held_file.clear();
        println!("[Discovery] Release");
    }
}

pub type SharedDiscoveryState = Arc<RwLock<DiscoveryState>>;

/// Start the beacon broadcaster (sends UDP every 1s)
pub async fn start_beacon(state: SharedDiscoveryState) {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Discovery] Failed to bind beacon socket: {}", e);
            return;
        }
    };

    if let Err(e) = socket.set_broadcast(true) {
        eprintln!("[Discovery] Failed to enable broadcast: {}", e);
        return;
    }

    println!("[Discovery] Beacon started, broadcasting every {}ms", BEACON_INTERVAL_MS);

    loop {
        let packet = {
            let state = state.read().await;
            BeaconPacket {
                id: state.device_id.clone(),
                ip: state.local_ip.clone(),
                name: state.device_name.clone(),
                is_holding: state.is_holding,
                held_file: state.held_file.clone(),
            }
        };

        if let Ok(json) = serde_json::to_string(&packet) {
            // Send to both broadcast and multicast for better compatibility
            let _ = socket.send_to(json.as_bytes(), BROADCAST_ADDR).await;
            let _ = socket.send_to(json.as_bytes(), MULTICAST_ADDR).await;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(BEACON_INTERVAL_MS)).await;
    }
}

/// Start the UDP listener (receives peer beacons)
pub async fn start_listener<F>(state: SharedDiscoveryState, on_peer: F)
where
    F: Fn(Peer, bool) + Send + Sync + 'static,
{
    let socket = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Discovery] Failed to bind listener on port {}: {}", DISCOVERY_PORT, e);
            eprintln!("[Discovery] This may be due to firewall or another process using the port.");
            return;
        }
    };

    println!("[Discovery] Listener started on port {}", DISCOVERY_PORT);

    let mut buf = [0u8; 4096];
    let on_peer = Arc::new(on_peer);

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _addr)) => {
                if let Ok(json_str) = std::str::from_utf8(&buf[..len]) {
                    if let Ok(packet) = serde_json::from_str::<BeaconPacket>(json_str) {
                        // Ignore our own broadcasts
                        let our_id = {
                            let state = state.read().await;
                            state.device_id.clone()
                        };

                        if packet.id == our_id {
                            continue;
                        }

                        let peer = Peer {
                            id: packet.id.clone(),
                            ip: packet.ip.clone(),
                            name: packet.name.clone(),
                            is_holding: packet.is_holding,
                            held_file: packet.held_file.clone(),
                        };

                        // Check if this is a new peer or grab update
                        let (is_new, is_grab_update) = {
                            let mut state = state.write().await;
                            let existing = state.peers.get(&peer.id);
                            let is_new = existing.is_none();
                            let is_grab_update = existing
                                .map(|p| p.is_holding != peer.is_holding || p.held_file != peer.held_file)
                                .unwrap_or(false);
                            
                            state.peers.insert(peer.id.clone(), peer.clone());
                            (is_new, is_grab_update)
                        };

                        if is_new {
                            println!("[Discovery] New peer: {} at {}", peer.name, peer.ip);
                            on_peer(peer, false);
                        } else if is_grab_update {
                            println!("[Discovery] Grab update from {}: holding={}", peer.name, peer.is_holding);
                            on_peer(peer, true);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Discovery] Receive error: {}", e);
            }
        }
    }
}
