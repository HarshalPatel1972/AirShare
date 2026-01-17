<script lang="ts">
  // CyberHand is now INVISIBLE - only moving the real OS cursor
  // This component is kept for potential future gesture feedback UI
  import { handState, isGrabbing } from '$lib/stores/handStore';
  
  // Track states for potential future visual feedback
  $: gesture = $handState.gesture;
  $: grabbing = $isGrabbing;
</script>

<!-- 
  No visual cursor rendered here.
  The real Windows cursor is moved by GestureController via simulate_mouse_move.
  
  Future enhancement: Could show gesture feedback icons here (e.g., scroll indicator)
-->
