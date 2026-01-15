<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { transferState, setRemoteGrab, clearRemoteGrab } from '$lib/stores/transferStore';
  import { handState, isHovering } from '$lib/stores/handStore';
  import { fly, fade, scale } from 'svelte/transition';

  let unlisten: UnlistenFn | null = null;

  interface GrabUpdate {
    id: string;
    ip: string;
    name: string;
    isHolding: boolean;
    heldFile: string;
  }

  onMount(async () => {
    // Listen for grab updates from peers
    unlisten = await listen<GrabUpdate>('grab-update', (event) => {
      const peer = event.payload;
      console.log('Grab update from peer:', peer);

      if (peer.isHolding && peer.heldFile) {
        setRemoteGrab(peer.id, peer.name, peer.ip, peer.heldFile);
      } else {
        clearRemoteGrab();
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  // Handle receiving file when user releases (open palm)
  $: if ($isHovering && $transferState.remoteGrab) {
    handleReceive();
  }

  async function handleReceive() {
    if (!$transferState.remoteGrab) return;

    const { peerName, peerIp, fileName } = $transferState.remoteGrab;

    console.log('Requesting transfer:', fileName);

    // Request transfer (goes through approval flow)
    try {
      await invoke('send_to_sidecar', { 
        command: `REQUEST_TRANSFER ${fileName} ${peerName} ${peerIp}` 
      });
      clearRemoteGrab();
    } catch (err) {
      console.error('Transfer request failed:', err);
    }
  }
</script>

{#if $transferState.remoteGrab}
  <div class="ghost-hand" in:fade={{ duration: 300 }} out:fade={{ duration: 200 }}>
    <div 
      class="ghost-icon"
      in:fly={{ y: -30, duration: 400 }}
      out:scale={{ duration: 200, start: 0.8 }}
      style="
        left: {$handState.cursorPosition.x * 100}%;
        top: {$handState.cursorPosition.y * 100}%;
      "
    >
      <div class="ghost-ring"></div>
      <div class="ghost-file">
        <span class="file-icon">📦</span>
        <span class="file-name">{$transferState.remoteGrab.fileName}</span>
      </div>
      <div class="ghost-label">
        From: {$transferState.remoteGrab.peerName}
      </div>
    </div>

    <div class="receive-hint" in:fly={{ y: 20, duration: 400, delay: 200 }}>
      🖐️ Open your palm to receive the file!
    </div>
  </div>
{/if}

<style>
  .ghost-hand {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 9998;
  }

  .ghost-icon {
    position: absolute;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    animation: float 2s ease-in-out infinite;
  }

  @keyframes float {
    0%, 100% { transform: translate(-50%, -50%) translateY(0); }
    50% { transform: translate(-50%, -50%) translateY(-10px); }
  }

  .ghost-ring {
    width: 80px;
    height: 80px;
    border: 3px dashed rgba(96, 165, 250, 0.6);
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { 
      transform: scale(1);
      opacity: 0.6;
    }
    50% { 
      transform: scale(1.1);
      opacity: 1;
    }
  }

  .ghost-file {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    background: rgba(0, 0, 0, 0.8);
    padding: 0.5rem;
    border-radius: 8px;
  }

  .file-icon {
    font-size: 1.5rem;
  }

  .file-name {
    font-size: 0.7rem;
    color: #60a5fa;
    max-width: 60px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ghost-label {
    margin-top: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: rgba(96, 165, 250, 0.3);
    border-radius: 4px;
    font-size: 0.7rem;
    color: #60a5fa;
    white-space: nowrap;
  }

  .receive-hint {
    position: fixed;
    bottom: 150px;
    left: 50%;
    transform: translateX(-50%);
    padding: 1rem 2rem;
    background: rgba(96, 165, 250, 0.2);
    border: 1px solid rgba(96, 165, 250, 0.5);
    border-radius: 12px;
    color: #60a5fa;
    font-size: 1.1rem;
    font-weight: 500;
    animation: bounce 1s ease-in-out infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateX(-50%) translateY(0); }
    50% { transform: translateX(-50%) translateY(-5px); }
  }
</style>
