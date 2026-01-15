<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { fly, fade } from 'svelte/transition';

  interface TransferRequest {
    filename: string;
    senderName: string;
    senderIp: string;
  }

  let pendingTransfer: TransferRequest | null = null;
  let unlisten: UnlistenFn | null = null;
  let autoRejectTimeout: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    // Listen for transfer requests from Go engine
    unlisten = await listen<TransferRequest>('transfer-request', (event) => {
      console.log('Transfer request:', event.payload);
      pendingTransfer = event.payload;
      
      // Auto-reject after 30 seconds
      autoRejectTimeout = setTimeout(() => {
        handleReject();
      }, 30000);
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (autoRejectTimeout) clearTimeout(autoRejectTimeout);
  });

  async function handleAccept() {
    if (!pendingTransfer) return;

    const downloadUrl = `http://${pendingTransfer.senderIp}:8080/file/${pendingTransfer.filename}`;
    const destPath = `C:/Users/Public/Downloads/${pendingTransfer.filename}`;

    try {
      await invoke('send_to_sidecar', { 
        command: `CONFIRM_TRANSFER ${downloadUrl} ${destPath}` 
      });
      console.log('Transfer accepted:', pendingTransfer.filename);
    } catch (err) {
      console.error('Accept failed:', err);
    }

    clearPending();
  }

  async function handleReject() {
    if (!pendingTransfer) return;

    try {
      await invoke('send_to_sidecar', { command: 'REJECT_TRANSFER' });
      console.log('Transfer rejected');
    } catch (err) {
      console.error('Reject error:', err);
    }

    clearPending();
  }

  function clearPending() {
    pendingTransfer = null;
    if (autoRejectTimeout) {
      clearTimeout(autoRejectTimeout);
      autoRejectTimeout = null;
    }
  }
</script>

{#if pendingTransfer}
  <div class="toast-overlay" transition:fade={{ duration: 200 }}>
    <div class="toast" transition:fly={{ y: -50, duration: 300 }}>
      <div class="toast-icon">📦</div>
      <div class="toast-content">
        <h4>Incoming File</h4>
        <p class="filename">{pendingTransfer.filename}</p>
        <p class="sender">from <strong>{pendingTransfer.senderName}</strong></p>
      </div>
      <div class="toast-actions">
        <button class="accept-btn" onclick={handleAccept}>
          ✓ Accept
        </button>
        <button class="reject-btn" onclick={handleReject}>
          ✕ Reject
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .toast-overlay {
    position: fixed;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10000;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.5rem;
    background: rgba(30, 30, 40, 0.95);
    border: 1px solid rgba(100, 108, 255, 0.5);
    border-radius: 16px;
    box-shadow: 
      0 8px 32px rgba(0, 0, 0, 0.4),
      0 0 40px rgba(100, 108, 255, 0.2);
    backdrop-filter: blur(10px);
  }

  .toast-icon {
    font-size: 2.5rem;
    animation: bounce 1s ease-in-out infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-5px); }
  }

  .toast-content {
    flex: 1;
  }

  h4 {
    margin: 0 0 0.25rem 0;
    font-size: 1rem;
    color: #fff;
  }

  .filename {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #60a5fa;
  }

  .sender {
    margin: 0.25rem 0 0 0;
    font-size: 0.85rem;
    color: #888;
  }

  .toast-actions {
    display: flex;
    gap: 0.5rem;
  }

  .accept-btn, .reject-btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .accept-btn {
    background: #4ade80;
    color: #000;
  }

  .accept-btn:hover {
    background: #22c55e;
    transform: scale(1.05);
  }

  .reject-btn {
    background: rgba(255, 100, 100, 0.2);
    color: #ff6b6b;
    border: 1px solid rgba(255, 100, 100, 0.4);
  }

  .reject-btn:hover {
    background: rgba(255, 100, 100, 0.3);
    transform: scale(1.05);
  }
</style>
