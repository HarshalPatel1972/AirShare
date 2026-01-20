<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import GestureController from '$lib/components/GestureController.svelte';
  import { handState } from '$lib/stores/handStore';
  
  // Mode: 'dashboard' or 'overlay'
  let mode = $state<'dashboard' | 'overlay'>('dashboard');
  let isGrabbing = $state(false);
  let heldFilename = $state('');
  let previousGesture = $state('None');
  let lastCopyTime = 0;
  let lastCancelTime = 0; // Prevent re-grab after cancel
  let lastDropTime = 0;   // Prevent re-grab after drop
  const DROP_DELAY = 250; // Faster drops
  let dimOverlay = false;
  
  // Enter overlay mode
  async function activateOverlay() {
    try {
      await invoke('enter_phantom_mode');
      invoke('clear_clipboard').catch(() => {}); // Start clean
      mode = 'overlay';
      previousGesture = 'None';
      isGrabbing = false;
      heldFilename = '';
      lastCancelTime = 0;
      lastDropTime = 0;
      document.body.style.background = 'transparent';
      document.documentElement.style.background = 'transparent';
    } catch (err) {
      console.error('Failed to enter overlay:', err);
    }
  }

  // Exit overlay mode
  async function exitOverlay() {
    try {
      await invoke('exit_phantom_mode');
      mode = 'dashboard';
      isGrabbing = false;
      heldFilename = '';
      document.body.style.background = '';
      document.documentElement.style.background = '';
    } catch (err) {
      console.error('Failed to exit overlay:', err);
    }
  }
  
  // Handle keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+Shift+B toggles overlay
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      if (mode === 'overlay') {
        exitOverlay();
      } else {
        activateOverlay();
      }
    }
    // ESC exits overlay
    if (e.key === 'Escape' && mode === 'overlay') {
      exitOverlay();
    }
  }

  let dropTimer: any = null;

  import { listen } from '@tauri-apps/api/event';

  // React to gesture changes in overlay mode
  $effect(() => {
    if (mode !== 'overlay') return;
    
    const gesture = $handState.gesture;
    
    if (gesture !== previousGesture) {
      console.log(`[Overlay] Gesture: ${previousGesture} → ${gesture}, isGrabbing: ${isGrabbing}`);
      
      // 1. GESTURE CHANGED CHECK
      if (dropTimer && gesture !== 'Open_Palm') {
        console.log('✊ Drop cancelled - gesture changed back');
        clearTimeout(dropTimer);
        dropTimer = null;
      }
      
      // 2. THUMB DOWN = CANCEL
      if (gesture === 'Thumb_Down' && isGrabbing) { /* ... existing cancel logic ... */
        console.log('👎 Grab cancelled by user');
        isGrabbing = false;
        heldFilename = '';
        lastCancelTime = Date.now();
        if (dropTimer) { clearTimeout(dropTimer); dropTimer = null; }
      }
      
      // 3. FIST = CTRL+CLICK + COPY
      else if (gesture === 'Closed_Fist' && !isGrabbing) {
          // ... existing grab logic ...
           console.log('✊ GRAB (Ctrl+Click -> Copy)');
           isGrabbing = true;
           heldFilename = 'Grabbing...'; 
           
           invoke('simulate_ctrl_click')
             .then(() => {
                setTimeout(() => {
                    invoke('simulate_copy')
                      .then(() => {
                          console.log('✅ Copied');
                          heldFilename = 'Holding Item'; 
                          lastCopyTime = Date.now();
                      })
                      .catch(err => {
                          console.error('❌ Copy failed:', err);
                          heldFilename = 'Error';
                      });
                }, 50);
             })
             .catch(console.error);
      }
      
      // 4. OPEN PALM = PASTE (Drop)
      else if (gesture === 'Open_Palm') {
        // CASE A: Local Grab Drop
        if (isGrabbing) {
            /* ... existing local drop logic ... */
           if (Date.now() - lastCopyTime < 500) {
               console.log('⏳ Too soon');
           } else if (!dropTimer) {
               console.log(`🖐️ Drop requested (Local)...`);
               dropTimer = setTimeout(() => {
                   console.log('✅ DROP (Local)');
                   invoke('simulate_click').then(() => {
                       setTimeout(() => {
                           invoke('simulate_paste').then(() => {
                               console.log('✅ Pasted!');
                               setTimeout(() => { invoke('clear_clipboard').catch(() => {}); }, 500);
                           });
                       }, 100);
                   });
                   isGrabbing = false;
                   heldFilename = '';
                   lastDropTime = Date.now();
                   dropTimer = null;
               }, DROP_DELAY);
           }
        } 
        // CASE B: Remote Drop (Check Peers)
        else {
             // Check if any peer is holding a file
             invoke<any[]>('get_peers').then((peers) => {
                 const holdingPeer = peers.find(p => p.isHolding && p.heldFile);
                 if (holdingPeer) {
                     console.log(`🖐️ DROP (Remote) from ${holdingPeer.name}`);
                     heldFilename = `Receiving ${holdingPeer.heldFile}...`;
                     isGrabbing = true; // Show UI
                     
                     // 1. Download
                     const url = `http://${holdingPeer.ip}:8080/file/${holdingPeer.heldFile}`;
                     invoke('get_airshare_downloads').then((dlDir) => {
                         const dest = `${dlDir}\\${holdingPeer.heldFile}`;
                         invoke('download_file', { url, destPath: dest })
                             .then(() => {
                                 console.log('✅ Downloaded');
                                 // 2. Set Clipboard
                                 invoke('set_clipboard_files', { paths: [dest] })
                                     .then(() => {
                                         // 3. Paste
                                         invoke('simulate_click').then(() => {
                                            setTimeout(() => {
                                                invoke('simulate_paste');
                                                isGrabbing = false;
                                                heldFilename = '';
                                            }, 200);
                                         });
                                     });
                             })
                             .catch(err => {
                                 console.error('Download failed:', err);
                                 heldFilename = 'Error';
                             });
                     });
                 }
             });
        }
      }
      
      previousGesture = gesture;
    }
  });

  
  import { updateHandState } from '$lib/stores/handStore';

  // Mobile Touch Logic
  let isMobile = false;
  let touchTimer: any = null;
  let lastTouchX = 0;
  let lastTouchY = 0;

  onMount(() => {
    isMobile = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
    window.addEventListener('keydown', handleKeydown);
    
    if (isMobile) {
        // Prevent default scrolling on mobile to allow gestures
        document.body.style.overflow = 'hidden';
        document.body.style.touchAction = 'none';
        
        // Add Touch Listeners
        window.addEventListener('touchstart', handleTouchStart, { passive: false });
        window.addEventListener('touchmove', handleTouchMove, { passive: false });
        window.addEventListener('touchend', handleTouchEnd);
    }
    
    let unlistenGrab: () => void;

    // Async setup for listeners
    (async () => {
        unlistenGrab = await listen('grab-update', (event: any) => {
            const peer = event.payload;
            console.log('[Event] Grab Update:', peer);
            if (peer.isHolding && peer.heldFile && mode === 'overlay') {
                 heldFilename = `Remote: ${peer.heldFile}`;
                 isGrabbing = true; 
            } else if (!peer.isHolding && heldFilename.startsWith('Remote:')) {
                 isGrabbing = false;
                 heldFilename = '';
            }
        });
    })();
    
    // Sync cleanup wrapper
    return () => {
        if (unlistenGrab) unlistenGrab();
        window.removeEventListener('keydown', handleKeydown);
        if (isMobile) {
            window.removeEventListener('touchstart', handleTouchStart);
            window.removeEventListener('touchmove', handleTouchMove);
            window.removeEventListener('touchend', handleTouchEnd);
        }
    };
  });

  // Touch Handlers
  function handleTouchStart(e: TouchEvent) {
      if (!isMobile) return;
      const touch = e.touches[0];
      lastTouchX = touch.clientX / window.innerWidth;
      lastTouchY = touch.clientY / window.innerHeight;
      
      // Update Hand State (Initial Touch = Open Palm/Hover)
      updateHandState(true, 'Open_Palm', lastTouchX, lastTouchY, 1.0);
      
      // Start Long Press Timer for GRAB
      touchTimer = setTimeout(() => {
          console.log('📱 Mobile Long Press -> GRAB');
          updateHandState(true, 'Closed_Fist', lastTouchX, lastTouchY, 1.0);
          
          // Trigger File Pick if not holding anything
          if (heldFilename === '') {
              triggerFilePick();
          }
      }, 300); // 300ms hold to grab
  }

  function handleTouchMove(e: TouchEvent) {
      if (!isMobile) return;
      e.preventDefault(); // Prevent scroll
      const touch = e.touches[0];
      lastTouchX = touch.clientX / window.innerWidth;
      lastTouchY = touch.clientY / window.innerHeight;
      
      // Keep current gesture (Fist if grabbing, Palm if hovering)
      const currentGesture = $handState.gesture; // Use store value
      updateHandState(true, currentGesture, lastTouchX, lastTouchY, 1.0);
  }

  function handleTouchEnd(e: TouchEvent) {
      if (!isMobile) return;
      if (touchTimer) clearTimeout(touchTimer);
      
      console.log('📱 Mobile Release -> DROP');
      // Release = Open Palm (Drop)
      updateHandState(true, 'Open_Palm', lastTouchX, lastTouchY, 1.0);
      
      // Reset to None after short delay
      setTimeout(() => {
          if (!isGrabbing) {
            updateHandState(false, 'None', 0.5, 0.5, 0);
          }
      }, 200);
  }
  
  onDestroy(() => {
    // window.removeEventListener is handled in onMount return
  });
  // Mobile File Picking
  let fileInput: HTMLInputElement;
  
  async function handleFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const file = target.files[0];
      const filename = file.name;
      
      try {
        // 1. Read file
        const buffer = await file.arrayBuffer();
        const bytes = Array.from(new Uint8Array(buffer));
        
        // 2. Save to "Shared" folder (so Server serves it)
        await invoke('save_received_file', { filename, data: bytes });
        
        // 3. Set Grab Status (Broadcast to LAN)
        await invoke('set_grab', { filename });
        
        // 4. Update UI
        isGrabbing = true;
        heldFilename = filename;
        mode = 'overlay'; // Show "Holding" UI
        
        console.log(`📱 Mobile Grab: ${filename}`);
      } catch (err) {
        console.error('Mobile grab failed:', err);
        heldFilename = 'Error';
      }
    }
  }

  function triggerFilePick() {
    fileInput?.click();
  }

</script>

<!-- Always active: Gesture Controller -->
<GestureController />

<!-- Hidden File Input for Mobile -->
<input 
  type="file" 
  style="display:none" 
  bind:this={fileInput} 
  onchange={handleFileSelect} 
/>

{#if mode === 'dashboard'}
  <!-- DASHBOARD MODE -->
  <div class="dashboard">
    <div class="content">
      <h1>AirShare</h1>
      <div class="subtitle">Gesture-based file transfer</div>
      
      <div class="status-card">
        <div class="status-row">
          <span class="label">Hand Detected</span>
          <span class="value" class:active={$handState.isHandDetected}>
            {$handState.isHandDetected ? '✅ Yes' : '❌ No'}
          </span>
        </div>
        <div class="status-row">
          <span class="label">Gesture</span>
          <span class="value gesture">{$handState.gesture || 'None'}</span>
        </div>
      </div>
      
      <div class="controls">
          <!-- Desktop Overlay -->
          <button class="activate-btn" onclick={activateOverlay}>
            <span class="icon">👋</span>
            Activate Overlay
          </button>
          
          <!-- Mobile Grab Button -->
          <button class="activate-btn mobile-btn" onclick={triggerFilePick}>
            <span class="icon">📱</span>
            Pick File to Grab
          </button>
      </div>
      
      <p class="hint">Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>B</kbd> to toggle</p>
    </div>
  </div>
  
{:else}
  <!-- OVERLAY MODE -->
  <div class="overlay-ui">
    <div class="gesture-indicator" class:grabbing={isGrabbing}>
      <span class="emoji">
        {#if isGrabbing}
          ✊
        {:else if $handState.gesture === 'Open_Palm'}
          🖐️
        {:else}
          👆
        {/if}
      </span>
      <span class="status-text">
        {#if isGrabbing}
          <div class="holding-label">Holding:</div>
          <div class="filename">{heldFilename || '...'}</div>
          <div class="action-hint">Open Palm on PC to Drop</div>
        {:else}
          {$handState.gesture}
        {/if}
      </span>
    </div>
    
    <div class="exit-hint">
        {#if isGrabbing}
           Boardcasting to LAN...
        {:else}
           <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>B</kbd> or <kbd>ESC</kbd> to exit
        {/if}
    </div>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
  }
  
  .dashboard {
    min-height: 100vh;
    background: linear-gradient(135deg, #0a0a0f 0%, #1a1a2e 50%, #0f0f1a 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'Inter', -apple-system, sans-serif;
    color: #fff;
  }
  
  .content {
    text-align: center;
    max-width: 400px;
  }
  
  h1 {
    font-size: 3rem;
    font-weight: 700;
    margin: 0 0 0.5rem;
    background: linear-gradient(90deg, #00ff88, #00ccff);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .subtitle {
    color: rgba(255,255,255,0.5);
    margin: 0 0 2rem;
    font-size: 1rem;
  }
  
  .status-card {
    background: rgba(255,255,255,0.03);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 1rem;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }
  
  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0;
  }
  
  .status-row:not(:last-child) {
    border-bottom: 1px solid rgba(255,255,255,0.06);
  }
  
  .label {
    color: rgba(255,255,255,0.6);
    font-size: 0.9rem;
  }
  
  .value {
    font-weight: 500;
    font-size: 0.9rem;
  }
  
  .value.active {
    color: #00ff88;
  }
  
  .value.gesture {
    color: #00ccff;
    font-family: monospace;
  }
  
  .activate-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    width: 100%;
    padding: 1rem;
    background: linear-gradient(90deg, #00ff88, #00ccff);
    border: none;
    border-radius: 0.75rem;
    color: #000;
    font-weight: 600;
    font-size: 1.1rem;
    cursor: pointer;
    transition: transform 0.2s, filter 0.2s;
  }
  
  .activate-btn:hover {
    transform: translateY(-2px);
    filter: brightness(1.1);
  }
  
  .activate-btn:active {
    transform: translateY(0);
  }
  
  .activate-btn .icon {
    font-size: 1.5rem;
  }
  
  .controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 100%;
  }

  .hint {
    margin-top: 2rem;
    color: rgba(255,255,255,0.3);
    font-size: 0.9rem;
  }
  
  .mobile-btn {
    background: linear-gradient(90deg, #ff9966, #ff5e62);
  }
  
  kbd {
    background: rgba(255,255,255,0.1);
    padding: 0.2rem 0.4rem;
    border-radius: 0.3rem;
    font-family: monospace;
    font-size: 0.8rem;
  }
  
  /* OVERLAY MODE */
  .overlay-ui {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none; /* Let clicks pass through */
    z-index: 1000;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 10vh;
  }
  
  .gesture-indicator {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 2rem;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(10px);
    border-radius: 2rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: white;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .gesture-indicator.grabbing {
    background: rgba(0, 255, 136, 0.2);
    border-color: rgba(0, 255, 136, 0.4);
    transform: scale(1.1);
  }
  
  .emoji {
    font-size: 2.5rem;
  }
  
  .status-text {
    font-size: 1.2rem;
    font-weight: 500;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  
  .holding-label {
    font-size: 0.8rem;
    color: #aaa;
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  
  .filename {
    font-size: 1.1rem;
    font-weight: 700;
    color: #fff;
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .action-hint {
    font-size: 0.8rem;
    color: #00ff88;
    margin-top: 2px;
  }
  
  .exit-hint {
    margin-top: 1rem;
    background: rgba(0,0,0,0.5);
    padding: 0.5rem 1rem;
    border-radius: 2rem;
    font-size: 0.9rem;
    color: rgba(255,255,255,0.7);
  }
</style>
