<script lang="ts">
  import { stagedFiles, stageFile, getFileColor, type StagedFile } from '$lib/stores/fileStore';
  
  let isDragOver = false;
  let dropZone: HTMLDivElement;
  
  // Format file size
  function formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
  }
  
  // Handle drag over
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragOver = true;
  }
  
  // Handle drag leave
  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
  }
  
  // Handle drop - get files from OS
  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragOver = false;
    
    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;
    
    // Process each dropped file
    for (const file of Array.from(files)) {
      // Get file path using webkitRelativePath or name
      // Note: For security, browsers don't expose full path
      // We'll use the file object and name
      const path = (file as any).path || file.name; // Tauri may expose .path
      
      console.log(`[Portal] Dropped: ${file.name} (${formatSize(file.size)})`);
      
      // Stage the file
      stageFile(path, file.name, file.size);
    }
  }
  
  // Reactive file list
  $: files = $stagedFiles as StagedFile[];
</script>

<div 
  class="portal-dock"
  class:drag-over={isDragOver}
  bind:this={dropZone}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  role="region"
  aria-label="File drop zone"
>
  <div class="portal-header">
    <span class="portal-icon">🌀</span>
    <span class="portal-title">Portal</span>
  </div>
  
  {#if files.length === 0}
    <div class="drop-hint">
      <div class="drop-icon">📁</div>
      <div class="drop-text">Drop files here</div>
    </div>
  {:else}
    <div class="file-list">
      {#each files as file (file.id)}
        <div class="file-item" style="--file-color: {getFileColor(file.extension)}">
          <div class="file-icon">
            {#if ['jpg', 'jpeg', 'png', 'gif', 'webp'].includes(file.extension.toLowerCase())}
              🖼️
            {:else if ['mp4', 'mov', 'avi', 'mkv'].includes(file.extension.toLowerCase())}
              🎬
            {:else if ['mp3', 'wav', 'ogg'].includes(file.extension.toLowerCase())}
              🎵
            {:else if ['pdf'].includes(file.extension.toLowerCase())}
              📄
            {:else if ['zip', 'rar', '7z'].includes(file.extension.toLowerCase())}
              📦
            {:else}
              📁
            {/if}
          </div>
          <div class="file-info">
            <div class="file-name">{file.name}</div>
            <div class="file-size">{formatSize(file.size)}</div>
          </div>
          <div class="file-status">
            {#if file.isGrabbed}
              ✊
            {:else if file.isTransferring}
              ⏳
            {:else}
              ✨
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .portal-dock {
    position: fixed;
    left: 20px;
    top: 50%;
    transform: translateY(-50%);
    width: 200px;
    max-height: 400px;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 212, 255, 0.3);
    border-radius: 16px;
    padding: 1rem;
    z-index: 10000;
    transition: all 0.3s ease;
    overflow: hidden;
  }
  
  .portal-dock.drag-over {
    border-color: rgba(0, 255, 136, 0.8);
    box-shadow: 0 0 30px rgba(0, 255, 136, 0.4);
    transform: translateY(-50%) scale(1.02);
  }
  
  .portal-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .portal-icon {
    font-size: 1.5rem;
    animation: spin 4s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .portal-title {
    font-size: 1rem;
    font-weight: 600;
    color: #00d4ff;
    text-transform: uppercase;
    letter-spacing: 2px;
  }
  
  .drop-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem 1rem;
    color: rgba(255, 255, 255, 0.5);
    text-align: center;
  }
  
  .drop-icon {
    font-size: 2rem;
    margin-bottom: 0.5rem;
    opacity: 0.6;
  }
  
  .drop-text {
    font-size: 0.875rem;
  }
  
  .file-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 300px;
    overflow-y: auto;
  }
  
  .file-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    border-left: 3px solid var(--file-color);
    transition: all 0.2s;
  }
  
  .file-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .file-icon {
    font-size: 1.25rem;
  }
  
  .file-info {
    flex: 1;
    min-width: 0;
  }
  
  .file-name {
    font-size: 0.75rem;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .file-size {
    font-size: 0.65rem;
    color: rgba(255, 255, 255, 0.5);
  }
  
  .file-status {
    font-size: 0.875rem;
  }
</style>
