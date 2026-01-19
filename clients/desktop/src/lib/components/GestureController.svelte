<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { FilesetResolver, GestureRecognizer, type GestureRecognizerResult } from '@mediapipe/tasks-vision';
  import { updateHandState, clearHandState, type GestureType } from '$lib/stores/handStore';

  let videoElement: HTMLVideoElement;
  let canvasElement: HTMLCanvasElement;
  let gestureRecognizer: GestureRecognizer | null = null;
  let animationFrameId: number;
  let isRunning = false;
  let errorMessage = '';
  let isLoading = true;
  let previousGesture: GestureType = 'None';
  
  // Advanced gesture state
  let wasPinching = false;
  let lastScrollY = 0.5;
  let lastMediaTrigger = 0;
  let lastClickTime = 0;
  let scrollAccumulator = 0;
  let lastScreenX = 0;
  let lastScreenY = 0;
  
  // Cursor smoothing (exponential moving average)
  let smoothX = 0.5;
  let smoothY = 0.5;
  const SMOOTHING = 0.6; // Higher = faster response (was 0.3, too slow)

  // Haptic feedback (uses Vibration API on supported browsers)
  function triggerHaptic(style: 'heavy' | 'light') {
    if ('vibrate' in navigator) {
      if (style === 'heavy') {
        navigator.vibrate(50); // 50ms vibration for grab
      } else {
        navigator.vibrate(20); // 20ms vibration for release
      }
    }
    console.log(`[Haptic] ${style}`);
  }

  // Map MediaPipe gesture names to our types
  function mapGesture(gestureName: string): GestureType {
    const gestureMap: Record<string, GestureType> = {
      'Closed_Fist': 'Closed_Fist',
      'Open_Palm': 'Open_Palm',
      'Pointing_Up': 'Pointing_Up',
      'Thumb_Down': 'Thumb_Down',
      'Thumb_Up': 'Thumb_Up',
      'Victory': 'Victory',
      'ILoveYou': 'ILoveYou'
    };
    return gestureMap[gestureName] || 'None';
  }
  
  // CUSTOM FIST DETECTION - More reliable than MediaPipe's gesture recognition
  // Checks if all fingers are curled (tips below MCP joints)
  function detectFist(landmarks: any[]): boolean {
    // Finger tip indices: Index=8, Middle=12, Ring=16, Pinky=20
    // MCP (knuckle) indices: Index=5, Middle=9, Ring=13, Pinky=17
    const tips = [8, 12, 16, 20];
    const mcps = [5, 9, 13, 17];
    
    let curledFingers = 0;
    for (let i = 0; i < 4; i++) {
      const tipY = landmarks[tips[i]].y;
      const mcpY = landmarks[mcps[i]].y;
      // If tip is BELOW mcp (higher Y value = lower on screen), finger is curled
      if (tipY > mcpY) {
        curledFingers++;
      }
    }
    
    // Fist = at least 3 fingers curled
    return curledFingers >= 3;
  }
  
  // CUSTOM PALM DETECTION - All fingers extended
  function detectOpenPalm(landmarks: any[]): boolean {
    const tips = [8, 12, 16, 20];
    const mcps = [5, 9, 13, 17];
    
    let extendedFingers = 0;
    for (let i = 0; i < 4; i++) {
      const tipY = landmarks[tips[i]].y;
      const mcpY = landmarks[mcps[i]].y;
      // If tip is ABOVE mcp (lower Y value), finger is extended
      if (tipY < mcpY) {
        extendedFingers++;
      }
    }
    
    // Open palm = at least 3 fingers extended
    return extendedFingers >= 3;
  }

  async function initializeGestureRecognizer() {
    try {
      isLoading = true;
      
      // Load MediaPipe vision WASM
      const vision = await FilesetResolver.forVisionTasks(
        'https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm'
      );

      // Create gesture recognizer with model from static folder
      gestureRecognizer = await GestureRecognizer.createFromOptions(vision, {
        baseOptions: {
          modelAssetPath: '/models/gesture_recognizer.task',
          delegate: 'GPU'
        },
        runningMode: 'VIDEO',
        numHands: 1
      });

      console.log('[GestureController] MediaPipe initialized successfully');
      isLoading = false;
    } catch (err) {
      console.error('[GestureController] Failed to initialize:', err);
      errorMessage = `Failed to load gesture model: ${err}`;
      isLoading = false;
    }
  }

  async function startWebcam() {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { 
          width: { ideal: 640 },
          height: { ideal: 480 },
          facingMode: 'user'
        }
      });

      videoElement.srcObject = stream;
      await videoElement.play();
      
      console.log('[GestureController] Webcam started');
      isRunning = true;
      detectGestures();
    } catch (err) {
      console.error('[GestureController] Webcam error:', err);
      errorMessage = `Camera access denied: ${err}`;
    }
  }

  function detectGestures() {
    if (!gestureRecognizer || !videoElement || !isRunning) return;

    const startTimeMs = performance.now();
    
    try {
      const results: GestureRecognizerResult = gestureRecognizer.recognizeForVideo(
        videoElement,
        startTimeMs
      );

      processResults(results);
    } catch (err) {
      console.error('[GestureController] Detection error:', err);
    }

    // Continue the loop
    animationFrameId = requestAnimationFrame(detectGestures);
  }

  function processResults(results: GestureRecognizerResult) {
    if (results.landmarks && results.landmarks.length > 0 && results.gestures && results.gestures.length > 0) {
      const landmarks = results.landmarks[0];
      
      // Use PALM CENTER for stable cursor
      const wrist = landmarks[0];
      const middleFingerBase = landmarks[9];
      const indexFingerTip = landmarks[8];
      const thumbTip = landmarks[4];
      
      // Palm center = average of wrist and middle finger base
      const palmX = (wrist.x + middleFingerBase.x) / 2;
      const palmY = (wrist.y + middleFingerBase.y) / 2;
      
      // === RANGE REMAPPING ===
      // Invert X for mirror
      const rawX = 1 - palmX;
      const rawY = palmY;
      
      // Remap X: palm doesn't reach full 0-1 range
      const X_MIN = 0.1;
      const X_MAX = 0.9;
      const remappedX = (rawX - X_MIN) / (X_MAX - X_MIN);
      
      // Remap Y: observed range is roughly 0.2-0.8
      const Y_MIN = 0.2;
      const Y_MAX = 0.8;
      const remappedY = (rawY - Y_MIN) / (Y_MAX - Y_MIN);
      
      // Apply smoothing to remapped values
      smoothX = smoothX + SMOOTHING * (remappedX - smoothX);
      smoothY = smoothY + SMOOTHING * (remappedY - smoothY);
      
      // Clamp to valid range
      const finalX = Math.max(0, Math.min(1, smoothX));
      const finalY = Math.max(0, Math.min(1, smoothY));
      
      // Map to PHYSICAL screen (multiply by devicePixelRatio for Windows scaling)
      const dpr = window.devicePixelRatio || 1;
      const screenWidth = Math.round(window.screen.width * dpr);
      const screenHeight = Math.round(window.screen.height * dpr);
      const screenX = Math.round(finalX * screenWidth);
      const screenY = Math.round(finalY * screenHeight);
      
      // DEBUG: Show actual physical screen size
      console.log(`[MOVE] screenX: ${screenX}, screenY: ${screenY} (physical: ${screenWidth}x${screenHeight}, DPR: ${dpr})`);
      
      const cursorX = smoothX;
      const cursorY = smoothY;
      
      // CUSTOM GESTURE DETECTION - More reliable than MediaPipe's built-in
      let gestureName: GestureType = 'None';
      
      if (detectFist(landmarks)) {
        gestureName = 'Closed_Fist';
      } else if (detectOpenPalm(landmarks)) {
        gestureName = 'Open_Palm';
      }
      
      // Also get MediaPipe's gesture for comparison
      const mpGesture = results.gestures[0][0];
      const mpGestureName = mpGesture.categoryName;
      const confidence = mpGesture.score;

      // DEBUG: Log both custom and MediaPipe detection
      console.log(`[GESTURE] Custom: "${gestureName}" | MediaPipe: "${mpGestureName}" (conf: ${confidence.toFixed(2)})`);

      // DISABLED: OS cursor movement - testing with visual cursor only
      // invoke('simulate_mouse_move', { x: screenX, y: screenY }).catch(() => {});
      lastScreenX = screenX;
      lastScreenY = screenY;

      // ====== ONLY GRAB & DROP - ALL OTHER GESTURES DISABLED ======
      
      // Log every gesture transition
      if (gestureName !== previousGesture) {
        console.log(`🎯 [Gesture Change] "${previousGesture}" → "${gestureName}"`);
        
        // GRAB: Closed Fist = COPY (Ctrl+C)
        if (gestureName === 'Closed_Fist') {
          console.log('🤜 FIST DETECTED - Sending Ctrl+C!');
          triggerHaptic('heavy');
          invoke('simulate_copy')
            .then(() => console.log('✅ COPY SUCCESS'))
            .catch((err) => console.error('❌ COPY FAILED:', err));
        }
        
        // DROP: Open Palm after Fist = PASTE (Ctrl+V)
        if (gestureName === 'Open_Palm' && previousGesture === 'Closed_Fist') {
          console.log('🖐️ OPEN PALM AFTER FIST - Sending Ctrl+V!');
          triggerHaptic('light');
          invoke('simulate_paste')
            .then(() => console.log('✅ PASTE SUCCESS'))
            .catch((err) => console.error('❌ PASTE FAILED:', err));
        }
        
        previousGesture = gestureName;
      }

      // PINCH CLICK DISABLED - Only grab/drop active
      /*
      const pinchDistance = Math.sqrt(
        Math.pow(indexFingerTip.x - thumbTip.x, 2) +
        Math.pow(indexFingerTip.y - thumbTip.y, 2)
      );
      const isPinching = pinchDistance < 0.06;
      const now = Date.now();
      if (isPinching && !wasPinching && (now - lastClickTime > 300)) {
        invoke('simulate_click').catch(console.error);
        lastClickTime = now;
      }
      wasPinching = isPinching;
      */

      // Convert landmarks to 3D format
      const landmarks3D = landmarks.map((lm: { x: number; y: number; z: number }) => ({
        x: lm.x, y: lm.y, z: lm.z
      }));

      updateHandState(true, gestureName, cursorX, cursorY, confidence, landmarks3D);
    } else {
      clearHandState();
    }
  }

  function stopWebcam() {
    isRunning = false;
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
    if (videoElement?.srcObject) {
      const stream = videoElement.srcObject as MediaStream;
      stream.getTracks().forEach(track => track.stop());
    }
  }

  onMount(async () => {
    await initializeGestureRecognizer();
    if (!errorMessage) {
      await startWebcam();
    }
  });

  onDestroy(() => {
    stopWebcam();
    if (gestureRecognizer) {
      gestureRecognizer.close();
    }
  });
</script>

<div class="gesture-controller">
  {#if isLoading}
    <div class="status loading">
      <div class="spinner"></div>
      <span>Loading gesture model...</span>
    </div>
  {:else if errorMessage}
    <div class="status error">
      <span>⚠️ {errorMessage}</span>
    </div>
  {:else}
    <div class="video-container">
      <video 
        bind:this={videoElement}
        class="webcam-feed"
        playsinline
        muted
      ></video>
      <canvas bind:this={canvasElement} class="overlay-canvas"></canvas>
    </div>
  {/if}
</div>

<style>
  .gesture-controller {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 999999;
  }

  .video-container {
    position: relative;
    width: 200px;
    height: 150px;
    border-radius: 12px;
    overflow: hidden;
    border: 2px solid rgba(0, 212, 255, 0.6);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .webcam-feed {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transform: scaleX(-1); /* Mirror the video */
  }

  .overlay-canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  .status {
    padding: 0.75rem 1rem;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .status.loading {
    background: rgba(100, 108, 255, 0.2);
    border: 1px solid rgba(100, 108, 255, 0.4);
    color: #a5a8ff;
  }

  .status.error {
    background: rgba(255, 100, 100, 0.2);
    border: 1px solid rgba(255, 100, 100, 0.4);
    color: #ffa5a5;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(100, 108, 255, 0.3);
    border-top-color: #646cff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
