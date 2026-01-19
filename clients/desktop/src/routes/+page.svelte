<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import GestureController from '$lib/components/GestureController.svelte';
  import { handState } from '$lib/stores/handStore';
  
  // Mode: 'dashboard' or 'overlay'
  let mode = $state<'dashboard' | 'overlay'>('dashboard');
  let isGrabbing = $state(false);
  let previousGesture = $state('None');
  let lastCopyTime = 0;
  let lastCancelTime = 0; // Prevent re-grab after cancel
  let lastDropTime = 0;   // Prevent re-grab after drop
  
  // Enter overlay mode
  async function activateOverlay() {
    try {
      await invoke('enter_phantom_mode');
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
  const DROP_DELAY = 400; // ms to hold open palm before dropping

  // React to gesture changes in overlay mode
  $effect(() => {
    if (mode !== 'overlay') return;
    
    const gesture = $handState.gesture;
    
    if (gesture !== previousGesture) {
      console.log(`[Overlay] Gesture: ${previousGesture} → ${gesture}, isGrabbing: ${isGrabbing}`);
      
      // 1. GESTURE CHANGED CHECK
      // If we were waiting to drop (Open_Palm) and gesture changed -> CANCEL DROP
      if (dropTimer && gesture !== 'Open_Palm') {
        console.log('✊ Drop cancelled - gesture changed back');
        clearTimeout(dropTimer);
        dropTimer = null;
      }
      
      // 2. THUMB DOWN = CANCEL (abort grab)
      if (gesture === 'Thumb_Down' && isGrabbing) {
        console.log('👎 Grab cancelled by user');
        isGrabbing = false;
        heldFilename = '';
        lastCancelTime = Date.now(); // Start cooldown
        if (dropTimer) {
           clearTimeout(dropTimer);
           dropTimer = null;
        }
        // Don't return, let previousGesture update
      }
      
      // 3. FIST = CLICK + COPY (select file then copy)
      // Only if not already grabbing AND not in cancel cooldown
      else if (gesture === 'Closed_Fist' && !isGrabbing) {
        const now = Date.now();
        if (now - lastCancelTime < 1500) {
           console.log('⏳ Cancel cooldown - ignoring grab');
        } else if (now - lastDropTime < 2000) {
           console.log('⏳ Drop cooldown - ignoring grab');
        } else {
           // Start Grab Process
           isGrabbing = true;
           heldFilename = 'Grabbing...'; 
           
           // 1. Clear Clipboard to avoid grabbing old content
           invoke('clear_clipboard')
             .then(() => {
               // Safety delay for clipboard to settle
               setTimeout(() => {
                   // 2. Click to select
                   invoke('simulate_click')
                     .then(() => {
                       // 3. Wait and Copy
                       setTimeout(() => {
                     invoke('simulate_copy')
                       .then(() => {
                          // 4. Verify we actually grabbed something
                          setTimeout(() => {
                            // If user cancelled in the meantime, abort
                            if (!isGrabbing) return;

                            invoke('get_clipboard_files')
                              .then((files: string[]) => {
                                if (files && files.length > 0) {
                                  // SUCCESS
                                  const path = files[0];
                                  heldFilename = path.split(/[/\\]/).pop() || path;
                                  lastCopyTime = Date.now();
                                  console.log('✅ Held file:', heldFilename);
                                } else {
                                  // FAILED - Nothing copied
                                  console.log('❌ Grab failed - empty clipboard (grabbed nothing?)');
                                  isGrabbing = false;
                                  heldFilename = '';
                                  // Optional: Show "Missed it" toast?
                                }
                              })
                              .catch(() => {
                                isGrabbing = false;
                                heldFilename = '';
                              });
                          }, 200);
                       })
                       .catch(err => {
                         console.error('❌ Copy failed:', err);
                         isGrabbing = false;
                         heldFilename = '';
                       });
                   }, 100);
                 })
                 .catch(() => {
                    isGrabbing = false;
                    heldFilename = '';
                 });
              }, 50);
             })
             .catch(err => {
                console.error('Failed to clear clipboard:', err);
                isGrabbing = false;
                heldFilename = '';
             });
        }
      }
      
      // 4. OPEN PALM = PASTE (release/drop)
      else if (gesture === 'Open_Palm' && isGrabbing) {
        // Prevent immediate paste (debounce 1s)
        if (Date.now() - lastCopyTime < 1000) {
           console.log('⏳ Paste cooldown - ignored');
        } else if (!dropTimer) {
          // Start confirmation timer
          console.log(`⏳ Drop initiated - waiting ${DROP_DELAY}ms...`);
          dropTimer = setTimeout(() => {
            console.log('✅ Drop confirmed!');
            isGrabbing = false;
            heldFilename = '';
            lastDropTime = Date.now(); // Start drop cooldown
            invoke('simulate_paste')
              .then(() => console.log('✅ Pasted!'))
              .catch(err => console.error('❌ Paste failed:', err));
            dropTimer = null;
          }, DROP_DELAY);
        }
      }
      
      previousGesture = gesture;
    }
  });

  
  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });
  
  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- Always active: Gesture Controller -->
<GestureController />

{#if mode === 'dashboard'}
  <!-- DASHBOARD MODE -->
  <div class="dashboard">
    <div class="content">
      <h1>AirShare</h1>
      <div class="subtitle">Gesture-based file transfer</div>
      
      <div class="status-card">
        <div class="status-row">
          <span class="label">Hand Detected</span>
          <span class="value" class:active={$handState.isPresent}>
            {$handState.isPresent ? '✅ Yes' : '❌ No'}
          </span>
        </div>
        <div class="status-row">
          <span class="label">Gesture</span>
          <span class="value gesture">{$handState.gesture || 'None'}</span>
        </div>
      </div>
      
      <button class="activate-btn" onclick={activateOverlay}>
        <span class="icon">👋</span>
        Activate Overlay
      </button>
      
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
          <div class="action-hint">Open Palm to Drop • 👎 to Cancel</div>
        {:else}
          {$handState.gesture}
        {/if}
      </span>
    </div>
    
    <div class="exit-hint">
      <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>B</kbd> or <kbd>ESC</kbd> to exit
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
  
  .hint {
    margin-top: 2rem;
    color: rgba(255,255,255,0.3);
    font-size: 0.9rem;
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
