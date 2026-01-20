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
  const SMOOTHING = 0.6; // Higher = faster response
  
  // Drag state for movable camera view
  let isDragging = false;
  let dragOffsetX = 0;
  let dragOffsetY = 0;
  let camPosX = $state(20); // Right offset
  let camPosY = $state(20); // Bottom offset
  
  function startDrag(e: MouseEvent) {
    isDragging = true;
    dragOffsetX = e.clientX + camPosX;
    dragOffsetY = e.clientY + camPosY;
    e.preventDefault();
  }
  
  function onDrag(e: MouseEvent) {
    if (!isDragging) return;
    camPosX = dragOffsetX - e.clientX;
    camPosY = dragOffsetY - e.clientY;
  }
  
  function stopDrag() {
    isDragging = false;
  }

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
    if (!isRunning) return;

    // If model or video not ready, just loop again
    if (!gestureRecognizer || !videoElement) {
       animationFrameId = requestAnimationFrame(detectGestures);
       return;
    }

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

      // Move OS cursor to follow hand
      invoke('simulate_mouse_move', { x: screenX, y: screenY }).catch(() => {});
      lastScreenX = screenX;
      lastScreenY = screenY;

      // Track gesture changes (copy/paste handled in +page.svelte)
      if (gestureName !== previousGesture) {
        console.log(`🎯 [Gesture] "${previousGesture}" → "${gestureName}"`);
        previousGesture = gestureName;
      }

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

  let isMobile = false;

  onMount(async () => {
    isMobile = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
    
    // Always add drag listeners
    window.addEventListener('mouseup', stopDrag);
    window.addEventListener('mousemove', onDrag);

    if (isMobile) {
         console.log("📱 Mobile Touch Mode Active");
         isLoading = false;
         isRunning = true;
         return;
    }

    // DESKTOP ONLY: 
    // 1. Start webcam IMMEDIATELY so user sees themselves
    startWebcam();

    // 2. Load model in parallel (don't block camera)
    try {
      // Add timeout to model loading
      const modelLoadPromise = initializeGestureRecognizer();
      const timeoutPromise = new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Model load timed out (15s)')), 15000)
      );
      
      await Promise.race([modelLoadPromise, timeoutPromise]);
    } catch (err) {
      console.error('Gesture model failed:', err);
      errorMessage = `Model error: ${err}`;
      isLoading = false;
    }
  });

  onDestroy(() => {
    stopWebcam();
    if (gestureRecognizer) {
      gestureRecognizer.close();
    }
  });
</script>

<div 
  class="gesture-controller" 
  style="right: {camPosX}px; bottom: {camPosY}px;"
  onmousedown={startDrag}
  role="application"
>
  <div class="video-container">
    <div class="drag-handle">⋮⋮ Drag to move</div>
    <video 
      bind:this={videoElement}
      class="webcam-feed"
      playsinline
      muted
    ></video>
    <canvas bind:this={canvasElement} class="overlay-canvas"></canvas>
    
    {#if isLoading}
      <div class="status-overlay">
        <div class="spinner"></div>
      </div>
    {:else if errorMessage}
      <div class="status-overlay error">
        <span>⚠️</span>
      </div>
    {/if}
  </div>
</div>

<!-- Global mouse events for drag -->
<svelte:window on:mousemove={onDrag} on:mouseup={stopDrag} />

<style>
  .gesture-controller {
    position: fixed;
    z-index: 999999;
    cursor: move;
  }

  .video-container {
    position: relative;
    width: 200px;
    height: 150px;
    border-radius: 12px;
    overflow: hidden;
    border: 2px solid rgba(0, 212, 255, 0.6);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    background: #000;
  }
  
  .status-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0,0,0,0.5);
    z-index: 20;
    pointer-events: none;
  }
  
  .status-overlay.error {
    background: rgba(255, 0, 0, 0.2);
    color: red;
    font-size: 2rem;
  }
  
  .drag-handle {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    padding: 4px;
    background: rgba(0, 0, 0, 0.7);
    color: rgba(255, 255, 255, 0.6);
    font-size: 10px;
    text-align: center;
    z-index: 10;
    cursor: move;
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
