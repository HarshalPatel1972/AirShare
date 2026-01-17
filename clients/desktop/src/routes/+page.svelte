<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import DeviceList from '$lib/components/DeviceList.svelte';
  import GestureController from '$lib/components/GestureController.svelte';
  import CyberHand from '$lib/components/CyberHand.svelte';
  import FileGallery from '$lib/components/FileGallery.svelte';
  import GhostHand from '$lib/components/GhostHand.svelte';
  import NotificationToast from '$lib/components/NotificationToast.svelte';
  import { handState } from '$lib/stores/handStore';
  
  // Mode: 'dashboard' (normal window) or 'phantom' (transparent overlay)
  let mode: 'dashboard' | 'phantom' = 'dashboard';
  
  // Hand tracking status
  $: isTracking = $handState.isHandDetected;
  $: currentGesture = $handState.gesture;
  
  async function enterPhantomMode() {
    try {
      await invoke('enter_phantom_mode');
      mode = 'phantom';
      // Enable transparent background
      document.body.style.background = 'transparent';
      document.documentElement.style.background = 'transparent';
    } catch (err) {
      console.error('Failed to enter phantom mode:', err);
    }
  }
  
  async function exitPhantomMode() {
    try {
      await invoke('exit_phantom_mode');
      mode = 'dashboard';
      // Restore background
      document.body.style.background = '';
      document.documentElement.style.background = '';
    } catch (err) {
      console.error('Failed to exit phantom mode:', err);
    }
  }
  
  // ESC key to exit phantom mode
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && mode === 'phantom') {
      exitPhantomMode();
    }
  }
  
  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });
  
  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- Always active: Gesture detection (hidden) -->
<GestureController />

{#if mode === 'phantom'}
  <!-- PHANTOM MODE -->
  <CyberHand />
  <GhostHand />
  <NotificationToast />
  
  <button class="exit-btn" onclick={exitPhantomMode} title="Exit (ESC)">
    ✕
  </button>
{:else}
  <!-- DASHBOARD -->
  <main class="dashboard">
    <div class="center-content">
      <!-- Status -->
      <div class="status-pill" class:active={isTracking}>
        <span class="dot"></span>
        {isTracking ? currentGesture : 'No hand'}
      </div>
      
      <!-- Logo -->
      <h1>AirShare</h1>
      
      <!-- Start Button -->
      <button class="start-btn" onclick={enterPhantomMode}>
        👻 Start
      </button>
      
      <!-- Gesture Guide (minimal) -->
      <div class="gestures">
        <span title="Click">👌</span>
        <span title="Scroll">✌️</span>
        <span title="Grab">✊</span>
        <span title="Play/Pause">👍</span>
      </div>
    </div>
    
    <div class="panels">
      <FileGallery />
      <DeviceList />
    </div>
  </main>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Inter', -apple-system, sans-serif;
  }
  
  .dashboard {
    min-height: 100vh;
    background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 100%);
    color: #fff;
    display: flex;
    flex-direction: column;
    padding: 2rem;
  }
  
  .center-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
  }
  
  .center-content h1 {
    font-size: 2.5rem;
    font-weight: 700;
    margin: 0;
    background: linear-gradient(90deg, #00d4ff, #00ff88);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .status-pill {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 1rem;
    background: rgba(100, 100, 100, 0.2);
    border: 1px solid rgba(100, 100, 100, 0.3);
    border-radius: 20px;
    font-size: 0.75rem;
    color: #888;
  }
  
  .status-pill.active {
    background: rgba(0, 255, 136, 0.15);
    border-color: rgba(0, 255, 136, 0.4);
    color: #00ff88;
  }
  
  .status-pill .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  
  .start-btn {
    padding: 1rem 3rem;
    font-size: 1.2rem;
    font-weight: 600;
    background: linear-gradient(135deg, #00ff88, #00d4ff);
    border: none;
    border-radius: 50px;
    color: #0f0f1a;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
    box-shadow: 0 4px 20px rgba(0, 255, 136, 0.3);
  }
  
  .start-btn:hover {
    transform: scale(1.05);
    box-shadow: 0 6px 30px rgba(0, 255, 136, 0.5);
  }
  
  .gestures {
    display: flex;
    gap: 1.5rem;
    font-size: 1.5rem;
    opacity: 0.6;
  }
  
  .gestures span {
    cursor: help;
  }
  
  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    flex: 1;
  }
  
  /* Phantom Mode */
  .exit-btn {
    position: fixed;
    top: 20px;
    right: 20px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #fff;
    font-size: 1.2rem;
    cursor: pointer;
    z-index: 99999;
    backdrop-filter: blur(10px);
    transition: all 0.2s;
  }
  
  .exit-btn:hover {
    background: rgba(255, 100, 100, 0.3);
    border-color: rgba(255, 100, 100, 0.5);
  }
</style>
