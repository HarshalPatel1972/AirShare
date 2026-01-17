import { writable, derived } from 'svelte/store';

export interface RemoteGrab {
  peerId: string;
  peerName: string;
  peerIp: string;
  fileName: string;
}

export interface TransferState {
  // Local grab state
  isLocalGrab: boolean;
  localFile: string | null;
  
  // Remote grab (peer is holding a file)
  remoteGrab: RemoteGrab | null;
  
  // Download in progress
  isDownloading: boolean;
  downloadProgress: number;
}

const initialState: TransferState = {
  isLocalGrab: false,
  localFile: null,
  remoteGrab: null,
  isDownloading: false,
  downloadProgress: 0
};

// Main transfer state store
export const transferState = writable<TransferState>(initialState);

// Start local grab
export function startLocalGrab(filename: string) {
  transferState.update((state) => ({
    ...state,
    isLocalGrab: true,
    localFile: filename
  }));
}

// End local grab
export function endLocalGrab() {
  transferState.update((state) => ({
    ...state,
    isLocalGrab: false,
    localFile: null
  }));
}

// Set remote grab (peer is holding file)
export function setRemoteGrab(peerId: string, peerName: string, peerIp: string, fileName: string) {
  transferState.update((state) => ({
    ...state,
    remoteGrab: { peerId, peerName, peerIp, fileName }
  }));
}

// Clear remote grab
export function clearRemoteGrab() {
  transferState.update((state) => ({
    ...state,
    remoteGrab: null
  }));
}

// Set download progress
export function setDownloadProgress(progress: number) {
  transferState.update((state) => ({
    ...state,
    isDownloading: progress > 0 && progress < 100,
    downloadProgress: progress
  }));
}

// Derived stores
export const hasRemoteGrab = derived(transferState, ($state) => $state.remoteGrab !== null);
export const hasLocalGrab = derived(transferState, ($state) => $state.isLocalGrab);
