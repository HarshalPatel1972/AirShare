<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import DeviceList from '$lib/components/DeviceList.svelte';
  import GestureController from '$lib/components/GestureController.svelte';
  import CyberHand from '$lib/components/CyberHand.svelte';
  import FileGallery from '$lib/components/FileGallery.svelte';
  import GhostHand from '$lib/components/GhostHand.svelte';
  import NotificationToast from '$lib/components/NotificationToast.svelte';
  
  let isPhantomMode = true;
  let clickThrough = false;
  
  // Toggle click-through mode
  async function toggleClickThrough() {
    try {
      clickThrough = !clickThrough;
      await invoke('set_click_through', { enabled: clickThrough });
    } catch (err) {
      console.error('Failed to toggle click-through:', err);
    }
  }
  
  // Keyboard shortcut: ESC to toggle click-through
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      toggleClickThrough();
    }
  }
  
  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });
  
  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- Gesture detection (webcam preview in corner) -->
<GestureController />

<!-- 3D CyberHand overlay -->
<CyberHand />

<!-- Ghost hand for receiving files from peers -->
<GhostHand />

<!-- Security toast for transfer requests -->
<NotificationToast />

{#if !clickThrough}
  <!-- Main UI (visible when click-through is disabled) -->
  <main class="container" class:phantom={isPhantomMode}>
    <div class="status-bar">
      <span class="status-dot"></span>
      <span>AirShare Active</span>
      <button class="toggle-btn" onclick={toggleClickThrough}>
        {clickThrough ? '🔓 Unlock UI' : '🔒 Lock UI (Click-Through)'}
      </button>
    </div>
    
    <div class="panels">
      <FileGallery />
      <DeviceList />
    </div>
  </main>
{:else}
  <!-- Minimal indicator when in click-through mode -->
  <div class="phantom-indicator">
    <span>👻 Phantom Mode</span>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent !important;
    overflow: hidden;
  }
  
  :global(html) {
    background: transparent !important;
  }
  
  .container {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    padding: 1rem;
    background: rgba(15, 15, 15, 0.85);
    backdrop-filter: blur(10px);
    color: #f6f6f6;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }
  
  .container.phantom {
    background: rgba(15, 15, 15, 0.6);
  }
  
  .status-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(0, 212, 255, 0.1);
    border-radius: 12px;
    border: 1px solid rgba(0, 212, 255, 0.3);
    margin-bottom: 1rem;
  }
  
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #4ade80;
    box-shadow: 0 0 8px #4ade80;
    animation: pulse 2s ease-in-out infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .toggle-btn {
    margin-left: auto;
    padding: 0.5rem 1rem;
    background: rgba(100, 108, 255, 0.2);
    border: 1px solid rgba(100, 108, 255, 0.4);
    border-radius: 8px;
    color: #a5a8ff;
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.2s ease;
  }
  
  .toggle-btn:hover {
    background: rgba(100, 108, 255, 0.3);
  }
  
  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    flex: 1;
  }
  
  .phantom-indicator {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    padding: 0.5rem 1rem;
    background: rgba(0, 255, 136, 0.2);
    border: 1px solid rgba(0, 255, 136, 0.4);
    border-radius: 20px;
    color: #00ff88;
    font-family: Inter, sans-serif;
    font-size: 0.875rem;
    pointer-events: none;
    z-index: 10000;
  }
</style>
