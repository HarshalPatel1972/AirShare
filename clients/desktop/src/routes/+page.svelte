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
  <!-- PHANTOM MODE: Transparent overlay with 3D hand -->
  <CyberHand />
  <GhostHand />
  <NotificationToast />
  
  <div class="phantom-controls">
    <button class="exit-btn" onclick={exitPhantomMode}>
      ⬅ Exit Phantom Mode (ESC)
    </button>
  </div>
{:else}
  <!-- DASHBOARD MODE: Normal window with panels -->
  <main class="dashboard">
    <header class="header">
      <h1>🚀 AirShare</h1>
      <p class="tagline">Gesture-Powered File Sharing</p>
      
      <!-- Hand Tracking Status -->
      <div class="tracking-status" class:active={isTracking}>
        <span class="dot"></span>
        <span>{isTracking ? `Tracking: ${currentGesture}` : 'No hand detected'}</span>
      </div>
    </header>
    
    <div class="hero">
      <div class="hero-content">
        <h2>Control Your Computer</h2>
        <p>Use hand gestures to click, scroll, and share files across devices</p>
        
        <button class="phantom-btn" onclick={enterPhantomMode}>
          <span class="icon">👻</span>
          <span>ENTER PHANTOM MODE</span>
        </button>
        
        <div class="gesture-hints">
          <div class="hint">✊ Grab = Hold File</div>
          <div class="hint">👌 Pinch = Click</div>
          <div class="hint">✌️ Victory = Scroll</div>
          <div class="hint">👍 Thumbs Up = Play/Pause</div>
        </div>
      </div>
    </div>
    
    <div class="panels">
      <FileGallery />
      <DeviceList />
    </div>
    
    <footer class="footer">
      <span>AirShare v1.0</span>
      <span>•</span>
      <span>Press ESC to exit Phantom Mode</span>
    </footer>
  </main>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }
  
  /* Dashboard Mode Styles */
  .dashboard {
    min-height: 100vh;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
    color: #f6f6f6;
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
  }
  
  .header {
    text-align: center;
    margin-bottom: 1rem;
  }
  
  .header h1 {
    font-size: 2rem;
    margin: 0;
    background: linear-gradient(90deg, #00d4ff, #00ff88);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .tagline {
    color: #888;
    margin: 0.25rem 0 0 0;
  }
  
  .tracking-status {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding: 0.35rem 0.75rem;
    background: rgba(255, 100, 100, 0.2);
    border: 1px solid rgba(255, 100, 100, 0.4);
    border-radius: 20px;
    font-size: 0.75rem;
    color: #ff6b6b;
  }
  
  .tracking-status.active {
    background: rgba(74, 222, 128, 0.2);
    border-color: rgba(74, 222, 128, 0.4);
    color: #4ade80;
  }
  
  .tracking-status .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
    animation: blink 1s infinite;
  }
  
  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }
  
  .hero {
    display: flex;
    justify-content: center;
    margin: 1rem 0;
  }
  
  .hero-content {
    text-align: center;
    padding: 2rem;
    background: rgba(0, 212, 255, 0.05);
    border: 1px solid rgba(0, 212, 255, 0.2);
    border-radius: 20px;
    max-width: 500px;
  }
  
  .hero-content h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
  }
  
  .hero-content p {
    color: #aaa;
    margin: 0 0 1.5rem 0;
  }
  
  .phantom-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    width: 100%;
    padding: 1rem 2rem;
    font-size: 1.25rem;
    font-weight: 700;
    background: linear-gradient(135deg, #00ff88, #00d4ff);
    border: none;
    border-radius: 12px;
    color: #1a1a2e;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 4px 20px rgba(0, 255, 136, 0.3);
  }
  
  .phantom-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 30px rgba(0, 255, 136, 0.5);
  }
  
  .phantom-btn .icon {
    font-size: 1.5rem;
  }
  
  .gesture-hints {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
    margin-top: 1.5rem;
    font-size: 0.875rem;
  }
  
  .hint {
    padding: 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    color: #aaa;
  }
  
  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    flex: 1;
    min-height: 0;
  }
  
  .footer {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    padding-top: 1rem;
    color: #666;
    font-size: 0.75rem;
  }
  
  /* Phantom Mode Styles */
  .phantom-controls {
    position: fixed;
    top: 20px;
    left: 20px;
    z-index: 10001;
  }
  
  .exit-btn {
    padding: 0.5rem 1rem;
    background: rgba(0, 0, 0, 0.7);
    border: 1px solid rgba(0, 212, 255, 0.5);
    border-radius: 20px;
    color: #00d4ff;
    cursor: pointer;
    font-size: 0.875rem;
    backdrop-filter: blur(10px);
    transition: all 0.2s ease;
  }
  
  .exit-btn:hover {
    background: rgba(0, 212, 255, 0.2);
  }
</style>
