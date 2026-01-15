<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { peers, addPeer, type Peer } from '$lib/stores/deviceStore';

  let unlisten: UnlistenFn | null = null;

  onMount(async () => {
    // Listen for peer-discovered events from Rust
    unlisten = await listen<Peer>('peer-discovered', (event) => {
      console.log('Peer discovered:', event.payload);
      addPeer(event.payload);
    });
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });
</script>

<div class="device-list">
  <h2>📡 Nearby Devices</h2>
  
  {#if $peers.length === 0}
    <div class="scanning">
      <div class="spinner"></div>
      <p>Scanning for devices...</p>
    </div>
  {:else}
    <ul class="peers">
      {#each $peers as peer (peer.id)}
        <li class="peer-card">
          <div class="peer-icon">💻</div>
          <div class="peer-info">
            <span class="peer-name">{peer.name}</span>
            <span class="peer-ip">{peer.ip}</span>
          </div>
          <div class="peer-status online"></div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .device-list {
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    margin: 1rem 0;
  }

  h2 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .scanning {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
    color: #888;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(100, 108, 255, 0.2);
    border-top-color: #646cff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .peers {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .peer-card {
    display: flex;
    align-items: center;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    transition: all 0.2s ease;
  }

  .peer-card:hover {
    background: rgba(255, 255, 255, 0.12);
    transform: translateY(-2px);
  }

  .peer-icon {
    font-size: 1.5rem;
    margin-right: 1rem;
  }

  .peer-info {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .peer-name {
    font-weight: 600;
    font-size: 1rem;
  }

  .peer-ip {
    font-size: 0.85rem;
    color: #888;
    font-family: monospace;
  }

  .peer-status {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin-left: 1rem;
  }

  .peer-status.online {
    background: #4ade80;
    box-shadow: 0 0 8px #4ade80;
  }
</style>
