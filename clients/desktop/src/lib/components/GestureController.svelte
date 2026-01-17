<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
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
      // Get the first hand's landmarks
      const landmarks = results.landmarks[0];
      
      // Use index finger tip (landmark 8) or wrist (landmark 0) for cursor position
      const indexFingerTip = landmarks[8];
      const wrist = landmarks[0];
      
      // Use a blend - finger tip for precision, wrist for stability
      const cursorX = indexFingerTip.x * 0.7 + wrist.x * 0.3;
      const cursorY = indexFingerTip.y * 0.7 + wrist.y * 0.3;

      // Get detected gesture
      const gesture = results.gestures[0][0];
      const gestureName = mapGesture(gesture.categoryName);
      const confidence = gesture.score;

      // Haptic feedback on gesture change
      if (gestureName !== previousGesture) {
        if (gestureName === 'Closed_Fist' && previousGesture !== 'Closed_Fist') {
          triggerHaptic('heavy'); // Grab
        } else if (gestureName === 'Open_Palm' && previousGesture === 'Closed_Fist') {
          triggerHaptic('light'); // Release
        }
        previousGesture = gestureName;
      }

      // Convert landmarks to 3D format { x, y, z }
      const landmarks3D = landmarks.map((lm: { x: number; y: number; z: number }) => ({
        x: lm.x,
        y: lm.y,
        z: lm.z
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
    z-index: 100;
    /* Hidden but still functional - webcam processes for gestures */
    opacity: 0;
    pointer-events: none;
    width: 1px;
    height: 1px;
    overflow: hidden;
  }

  .video-container {
    position: relative;
    width: 160px;
    height: 120px;
    border-radius: 12px;
    overflow: hidden;
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
