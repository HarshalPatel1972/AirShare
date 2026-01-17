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
      
      // Use index finger tip for cursor
      const indexFingerTip = landmarks[8];
      const thumbTip = landmarks[4];
      
      // === SCALED SCREEN MAPPING ===
      // The hand typically only moves within 20-80% of camera view
      // Scale up to allow full screen coverage with smaller movements
      const SCALE = 2.5;  // Movement multiplier
      const CENTER_X = 0.5;
      const CENTER_Y = 0.5;
      
      // Invert X for mirror, then scale from center
      const rawX = 1 - indexFingerTip.x;
      const rawY = indexFingerTip.y;
      
      // Scale movements from center point
      const scaledX = CENTER_X + (rawX - CENTER_X) * SCALE;
      const scaledY = CENTER_Y + (rawY - CENTER_Y) * SCALE;
      
      // Clamp to 0-1
      const normalizedX = Math.max(0, Math.min(1, scaledX));
      const normalizedY = Math.max(0, Math.min(1, scaledY));
      
      const cursorX = normalizedX;
      const cursorY = normalizedY;
      
      // Get detected gesture
      const gesture = results.gestures[0][0];
      const gestureName = mapGesture(gesture.categoryName);
      const confidence = gesture.score;

      // === MOVE REAL OS CURSOR ===
      const screenWidth = window.screen.width;
      const screenHeight = window.screen.height;
      const screenX = Math.round(normalizedX * screenWidth);
      const screenY = Math.round(normalizedY * screenHeight);
      
      // Debug log
      console.log(`Hand: (${rawX.toFixed(2)}, ${rawY.toFixed(2)}) → Screen: (${screenX}, ${screenY})`);
      
      // Move cursor (less throttle for smoother movement)
      if (Math.abs(screenX - lastScreenX) > 2 || Math.abs(screenY - lastScreenY) > 2) {
        invoke('simulate_mouse_move', { x: screenX, y: screenY }).catch(() => {});
        lastScreenX = screenX;
        lastScreenY = screenY;
      }

      // === PINCH DETECTION (threshold increased for reliability) ===
      const pinchDistance = Math.sqrt(
        Math.pow(indexFingerTip.x - thumbTip.x, 2) +
        Math.pow(indexFingerTip.y - thumbTip.y, 2)
      );
      
      const isPinching = pinchDistance < 0.06; // Slightly larger threshold
      
      // Pinch click with debounce
      const now = Date.now();
      if (isPinching && !wasPinching && (now - lastClickTime > 300)) {
        invoke('simulate_click').catch(console.error);
        lastClickTime = now;
      }
      wasPinching = isPinching;

      // === VICTORY SCROLL (improved sensitivity) ===
      if (gestureName === 'Victory') {
        const deltaY = lastScrollY - cursorY;
        
        // Accumulate scroll and trigger when threshold reached
        scrollAccumulator += deltaY * 100; // Scale up for sensitivity
        
        if (Math.abs(scrollAccumulator) > 3) {
          const scrollAmount = Math.round(scrollAccumulator);
          invoke('simulate_scroll', { direction: scrollAmount }).catch(() => {});
          scrollAccumulator = 0;
        }
        lastScrollY = cursorY;
      } else {
        lastScrollY = cursorY;
        scrollAccumulator = 0;
      }

      // === THUMBS UP MEDIA (2s debounce) ===
      if (gestureName === 'Thumb_Up' && (now - lastMediaTrigger > 2000)) {
        invoke('simulate_media_toggle').catch(console.error);
        lastMediaTrigger = now;
      }

      // Haptic feedback on gesture change
      if (gestureName !== previousGesture) {
        if (gestureName === 'Closed_Fist') triggerHaptic('heavy');
        else if (gestureName === 'Open_Palm' && previousGesture === 'Closed_Fist') triggerHaptic('light');
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
