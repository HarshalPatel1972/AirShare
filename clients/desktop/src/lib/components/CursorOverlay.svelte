<script lang="ts">
  import { handState, isGrabbing, isHovering } from '$lib/stores/handStore';
</script>

<div class="cursor-overlay">
  {#if $handState.isHandDetected}
    <div 
      class="cursor"
      class:grabbing={$isGrabbing}
      class:hovering={$isHovering}
      style="
        left: {$handState.cursorPosition.x * 100}%;
        top: {$handState.cursorPosition.y * 100}%;
      "
    >
      <div class="cursor-ring"></div>
      <div class="cursor-dot"></div>
    </div>

    <!-- Gesture indicator -->
    <div class="gesture-label">
      {$handState.gesture}
    </div>
  {/if}
</div>

<style>
  .cursor-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 9999;
  }

  .cursor {
    position: absolute;
    transform: translate(-50%, -50%);
    transition: all 0.05s ease-out;
  }

  .cursor-ring {
    width: 50px;
    height: 50px;
    border: 3px solid rgba(255, 255, 255, 0.8);
    border-radius: 50%;
    box-shadow: 
      0 0 20px rgba(255, 255, 255, 0.4),
      inset 0 0 20px rgba(255, 255, 255, 0.1);
    transition: all 0.15s ease;
  }

  .cursor-dot {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 10px;
    height: 10px;
    background: white;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.8);
    transition: all 0.15s ease;
  }

  /* Hovering state - White Ring (Open Palm) */
  .cursor.hovering .cursor-ring {
    border-color: rgba(255, 255, 255, 0.9);
    box-shadow: 
      0 0 30px rgba(255, 255, 255, 0.5),
      inset 0 0 20px rgba(255, 255, 255, 0.15);
  }

  .cursor.hovering .cursor-dot {
    background: white;
    box-shadow: 0 0 15px rgba(255, 255, 255, 1);
  }

  /* Grabbing state - Green Dot (Closed Fist) */
  .cursor.grabbing .cursor-ring {
    width: 30px;
    height: 30px;
    border-color: #4ade80;
    border-width: 4px;
    box-shadow: 
      0 0 30px rgba(74, 222, 128, 0.6),
      inset 0 0 15px rgba(74, 222, 128, 0.2);
  }

  .cursor.grabbing .cursor-dot {
    width: 14px;
    height: 14px;
    background: #4ade80;
    box-shadow: 0 0 20px rgba(74, 222, 128, 1);
  }

  .gesture-label {
    position: fixed;
    bottom: 150px;
    right: 20px;
    padding: 0.5rem 1rem;
    background: rgba(0, 0, 0, 0.7);
    border-radius: 8px;
    color: white;
    font-size: 0.875rem;
    font-family: monospace;
  }
</style>
