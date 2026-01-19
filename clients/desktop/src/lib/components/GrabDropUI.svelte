<script lang="ts">
  import { handState } from '$lib/stores/handStore';
  
  // File items
  interface FileItem {
    id: number;
    name: string;
    icon: string;
    zone: 'source' | 'target';
  }
  
  let files = $state<FileItem[]>([
    { id: 1, name: 'Photo.jpg', icon: '🖼️', zone: 'source' },
    { id: 2, name: 'Document.pdf', icon: '📄', zone: 'source' },
    { id: 3, name: 'Music.mp3', icon: '🎵', zone: 'source' },
  ]);
  
  // Grab state
  let grabbedFileId = $state<number | null>(null);
  let previousGesture = $state('None');
  
  // Cursor position (0-100 scale)
  let cursorX = $state(50);
  let cursorY = $state(50);
  
  // Which zone is cursor in
  let cursorZone = $derived(cursorX < 50 ? 'source' : 'target');
  
  // React to hand state changes
  $effect(() => {
    const gesture = $handState.gesture;
    // INVERT X - webcam is mirrored
    cursorX = (1 - $handState.cursorPosition.x) * 100;
    cursorY = $handState.cursorPosition.y * 100;
    
    if (gesture !== previousGesture) {
      // GRAB: Closed Fist - find file at cursor Y position
      if (gesture === 'Closed_Fist' && grabbedFileId === null) {
        const zoneFiles = files.filter(f => f.zone === cursorZone);
        
        if (zoneFiles.length > 0) {
          // Calculate which file based on cursor Y
          const fileAreaStart = 25; // Files start at 25% Y
          const fileRowHeight = 12; // Each file takes ~12% Y
          const fileAreaEnd = fileAreaStart + (zoneFiles.length * fileRowHeight);
          
          // Only grab if cursor is within file area
          if (cursorY >= fileAreaStart && cursorY <= fileAreaEnd) {
            const relativeY = cursorY - fileAreaStart;
            const fileIndex = Math.floor(relativeY / fileRowHeight);
            const clampedIndex = Math.max(0, Math.min(fileIndex, zoneFiles.length - 1));
            
            grabbedFileId = zoneFiles[clampedIndex].id;
            console.log(`[GRAB] Index: ${clampedIndex}, cursorY: ${cursorY.toFixed(0)}, file: ${zoneFiles[clampedIndex].name}`);
          } else {
            console.log(`[NO GRAB] Cursor outside file area. Y: ${cursorY.toFixed(0)}, range: ${fileAreaStart}-${fileAreaEnd}`);
          }
        }
      }
      
      // DROP: Open Palm releases file to current zone
      if (gesture === 'Open_Palm' && previousGesture === 'Closed_Fist' && grabbedFileId !== null) {
        const file = files.find(f => f.id === grabbedFileId);
        if (file) {
          file.zone = cursorZone;
          files = [...files];
        }
        grabbedFileId = null;
      }
      
      previousGesture = gesture;
    }
  });
  
  function reset() {
    files = [
      { id: 1, name: 'Photo.jpg', icon: '🖼️', zone: 'source' },
      { id: 2, name: 'Document.pdf', icon: '📄', zone: 'source' },
      { id: 3, name: 'Music.mp3', icon: '🎵', zone: 'source' },
    ];
    grabbedFileId = null;
  }
  
  // Get files for a zone
  function getFiles(zone: 'source' | 'target') {
    return files.filter(f => f.zone === zone);
  }
  
  // Get grabbed file
  $effect.pre(() => {
    // This just ensures reactivity
  });
</script>

<div class="container">
  <!-- Header -->
  <header>
    <h1>AirShare</h1>
    <div class="status">
      <span class="gesture" class:active={$handState.gesture === 'Closed_Fist'}>
        {$handState.gesture}
      </span>
      {#if grabbedFileId}
        <span class="holding">
          Holding: {files.find(f => f.id === grabbedFileId)?.name}
        </span>
      {/if}
    </div>
    <button onclick={reset}>Reset</button>
  </header>
  
  <!-- Panels -->
  <div class="panels">
    <!-- Source Panel -->
    <div class="panel" class:active={cursorZone === 'source'}>
      <div class="panel-header">
        <span class="icon">📁</span>
        <span>Source</span>
      </div>
      <div class="files">
        {#each getFiles('source') as file (file.id)}
          <div class="file" class:grabbed={grabbedFileId === file.id}>
            <span class="file-icon">{file.icon}</span>
            <span class="file-name">{file.name}</span>
          </div>
        {/each}
        {#if getFiles('source').length === 0}
          <div class="empty">No files</div>
        {/if}
      </div>
    </div>
    
    <!-- Target Panel -->
    <div class="panel" class:active={cursorZone === 'target'}>
      <div class="panel-header">
        <span class="icon">📥</span>
        <span>Target</span>
      </div>
      <div class="files">
        {#each getFiles('target') as file (file.id)}
          <div class="file" class:grabbed={grabbedFileId === file.id}>
            <span class="file-icon">{file.icon}</span>
            <span class="file-name">{file.name}</span>
          </div>
        {/each}
        {#if getFiles('target').length === 0}
          <div class="empty">Drop files here</div>
        {/if}
      </div>
    </div>
  </div>
  
  <!-- Hand Cursor -->
  <div 
    class="hand-cursor" 
    class:grabbing={grabbedFileId !== null}
    style="left: {cursorX}%; top: {cursorY}%;"
  >
    {#if grabbedFileId}
      <div class="grabbed-preview">
        {files.find(f => f.id === grabbedFileId)?.icon}
      </div>
    {/if}
  </div>
  
  <!-- Instructions -->
  <footer>
    <p><kbd>Fist</kbd> to grab • <kbd>Palm</kbd> to drop • Move cursor between panels</p>
  </footer>
</div>

<style>
  .container {
    min-height: 100vh;
    background: linear-gradient(135deg, #0a0a0f 0%, #1a1a2e 50%, #0f0f1a 100%);
    color: #fff;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    padding: 2rem;
    position: relative;
    overflow: hidden;
  }
  
  header {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    margin-bottom: 2rem;
  }
  
  h1 {
    font-size: 1.5rem;
    font-weight: 600;
    background: linear-gradient(90deg, #00ff88, #00ccff);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0;
  }
  
  .status {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 1;
  }
  
  .gesture {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 2rem;
    font-size: 0.875rem;
    font-weight: 500;
  }
  
  .gesture.active {
    background: rgba(0, 255, 136, 0.2);
    border-color: #00ff88;
    color: #00ff88;
  }
  
  .holding {
    font-size: 0.875rem;
    color: #00ff88;
    animation: pulse 1s infinite;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }
  
  button {
    padding: 0.5rem 1.25rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.5rem;
    color: #fff;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  button:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }
  
  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    height: calc(100vh - 200px);
  }
  
  .panel {
    background: rgba(255, 255, 255, 0.03);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.5rem;
    padding: 1.5rem;
    transition: all 0.3s;
  }
  
  .panel.active {
    border-color: rgba(0, 255, 136, 0.3);
    box-shadow: 0 0 40px rgba(0, 255, 136, 0.1);
  }
  
  .panel-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  
  .panel-header .icon {
    font-size: 1.25rem;
  }
  
  .panel-header span:last-child {
    font-size: 1rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
  }
  
  .files {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .file {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.25rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 1rem;
    transition: all 0.2s;
  }
  
  .file:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  
  .file.grabbed {
    background: rgba(0, 255, 136, 0.15);
    border-color: rgba(0, 255, 136, 0.4);
    transform: scale(1.02);
    box-shadow: 0 8px 32px rgba(0, 255, 136, 0.2);
  }
  
  .file-icon {
    font-size: 1.5rem;
  }
  
  .file-name {
    font-size: 0.9rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
  }
  
  .empty {
    text-align: center;
    padding: 3rem;
    color: rgba(255, 255, 255, 0.3);
    font-size: 0.9rem;
    border: 2px dashed rgba(255, 255, 255, 0.1);
    border-radius: 1rem;
  }
  
  .hand-cursor {
    position: fixed;
    width: 40px;
    height: 40px;
    background: radial-gradient(circle, rgba(255,255,255,0.3) 0%, transparent 70%);
    border: 2px solid rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    z-index: 1000;
    transition: all 0.1s;
  }
  
  .hand-cursor.grabbing {
    background: radial-gradient(circle, rgba(0,255,136,0.4) 0%, transparent 70%);
    border-color: #00ff88;
    width: 50px;
    height: 50px;
  }
  
  .grabbed-preview {
    position: absolute;
    top: -30px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 1.5rem;
    animation: float 1s infinite ease-in-out;
  }
  
  @keyframes float {
    0%, 100% { transform: translateX(-50%) translateY(0); }
    50% { transform: translateX(-50%) translateY(-5px); }
  }
  
  footer {
    position: fixed;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    text-align: center;
  }
  
  footer p {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.4);
  }
  
  kbd {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 0.25rem;
    font-family: inherit;
    font-size: 0.75rem;
  }
</style>
