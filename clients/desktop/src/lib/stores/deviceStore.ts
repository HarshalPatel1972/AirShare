import { writable } from 'svelte/store';

export interface Peer {
  id: string;
  ip: string;
  name: string;
}

// Store for discovered peers
export const peers = writable<Peer[]>([]);

// Add a new peer (prevents duplicates)
export function addPeer(peer: Peer) {
  peers.update((currentPeers) => {
    // Check if peer already exists
    const exists = currentPeers.some((p) => p.id === peer.id);
    if (exists) {
      return currentPeers;
    }
    return [...currentPeers, peer];
  });
}

// Remove a peer by ID
export function removePeer(peerId: string) {
  peers.update((currentPeers) => currentPeers.filter((p) => p.id !== peerId));
}

// Clear all peers
export function clearPeers() {
  peers.set([]);
}
