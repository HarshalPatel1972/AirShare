<script lang="ts">
  import GestureController from '$lib/components/GestureController.svelte';
  import GrabDropUI from '$lib/components/GrabDropUI.svelte';
</script>

<!-- Hidden: Gesture detection -->
<GestureController />

<!-- Main UI -->
<GrabDropUI />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #0a0a0f;
  }
</style>
