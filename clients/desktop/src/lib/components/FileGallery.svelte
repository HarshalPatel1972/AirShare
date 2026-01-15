<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { handState, isGrabbing, isHovering } from '$lib/stores/handStore';
  import { transferState, startLocalGrab, endLocalGrab } from '$lib/stores/transferStore';

  // Demo files for the gallery
  const demoFiles = [
    { id: 'demo1', name: 'photo.jpg', icon: '🖼️', color: '#4ade80' },
    { id: 'demo2', name: 'document.pdf', icon: '📄', color: '#60a5fa' },
    { id: 'demo3', name: 'music.mp3', icon: '🎵', color: '#f472b6' }
  ];

  let fileElements: Map<string, HTMLElement> = new Map();
  let hoveredFile: string | null = null;
  let grabbedFile: string | null = null;

  // Check if cursor is over a file
  function checkCursorOverFiles() {
    if (!$handState.isHandDetected) {
      hoveredFile = null;
      return;
    }

    const cursorX = $handState.cursorPosition.x * window.innerWidth;
    const cursorY = $handState.cursorPosition.y * window.innerHeight;

    hoveredFile = null;

    for (const [fileId, element] of fileElements) {
      const rect = element.getBoundingClientRect();
      if (
        cursorX >= rect.left &&
        cursorX <= rect.right &&
        cursorY >= rect.top &&
        cursorY <= rect.bottom
      ) {
        hoveredFile = fileId;
        break;
      }
    }
  }

  // Handle grab detection
  $: {
    checkCursorOverFiles();

    // If making a fist while hovering over a file, grab it
    if ($isGrabbing && hoveredFile && !grabbedFile) {
      const file = demoFiles.find(f => f.id === hoveredFile);
      if (file) {
        grabbedFile = hoveredFile;
        startLocalGrab(file.name);
        // Send GRAB command to Go engine
        invoke('send_to_sidecar', { command: `GRAB ${file.name}` }).catch(console.error);
        console.log('Grabbed:', file.name);
      }
    }

    // If opening palm while holding a file, release it
    if ($isHovering && grabbedFile) {
      endLocalGrab();
      // Send RELEASE command to Go engine
      invoke('send_to_sidecar', { command: 'RELEASE' }).catch(console.error);
      console.log('Released file');
      grabbedFile = null;
    }
  }

  function bindFileRef(el: HTMLElement, id: string) {
    fileElements.set(id, el);
    return {
      destroy() {
        fileElements.delete(id);
      }
    };
  }
</script>

<div class="file-gallery">
  <h3>📁 Shared Files</h3>
  <div class="file-grid">
    {#each demoFiles as file (file.id)}
      <div 
        class="file-card"
        class:hovered={hoveredFile === file.id}
        class:grabbed={grabbedFile === file.id}
        style="--accent-color: {file.color}"
        use:bindFileRef={file.id}
      >
        <div class="file-icon">{file.icon}</div>
        <span class="file-name">{file.name}</span>
        {#if grabbedFile === file.id}
          <div class="grabbed-indicator">✊ Holding</div>
        {/if}
      </div>
    {/each}
  </div>
  
  {#if $transferState.isLocalGrab}
    <div class="grab-status">
      ✊ Holding: {$transferState.localFile}
    </div>
  {/if}
</div>

<style>
  .file-gallery {
    margin: 1.5rem 0;
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  h3 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .file-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .file-card {
    padding: 1.5rem 1rem;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    border: 2px solid transparent;
    text-align: center;
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .file-card:hover,
  .file-card.hovered {
    background: rgba(255, 255, 255, 0.12);
    border-color: var(--accent-color, #646cff);
    transform: translateY(-4px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.3);
  }

  .file-card.grabbed {
    background: rgba(74, 222, 128, 0.2);
    border-color: #4ade80;
    transform: scale(0.95);
    box-shadow: 0 0 30px rgba(74, 222, 128, 0.4);
  }

  .file-icon {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  .file-name {
    display: block;
    font-size: 0.875rem;
    color: #ccc;
    word-break: break-all;
  }

  .grabbed-indicator {
    margin-top: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: #4ade80;
    color: #000;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .grab-status {
    margin-top: 1rem;
    padding: 0.75rem 1rem;
    background: rgba(74, 222, 128, 0.2);
    border: 1px solid #4ade80;
    border-radius: 8px;
    color: #4ade80;
    text-align: center;
    font-weight: 500;
  }
</style>
