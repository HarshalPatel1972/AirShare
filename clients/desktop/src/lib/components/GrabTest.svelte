<script lang="ts">
  import { handState } from '$lib/stores/handStore';
  
  // Items with POSITIONS (x, y in percentage 0-100)
  let items = $state([
    { id: 1, name: '📷 Photo', x: 25, y: 50, grabbed: false },
    { id: 2, name: '📄 Doc', x: 50, y: 50, grabbed: false },
    { id: 3, name: '🎵 Music', x: 75, y: 50, grabbed: false },
  ]);
  
  // Cursor position (0-100 scale to match items)
  let cursorX = $state(50);
  let cursorY = $state(50);
  let grabbedItemId = $state<number | null>(null);
  let previousGesture = $state('None');
  let gestureLog = $state<string[]>([]);
  
  // Subscribe to hand state
  $effect(() => {
    const gesture = $handState.gesture;
    // Convert 0-1 to 0-100 for easier positioning
    cursorX = $handState.cursorPosition.x * 100;
    cursorY = $handState.cursorPosition.y * 100;
    
    // If holding an item, move it with cursor
    if (grabbedItemId !== null) {
      const item = items.find(i => i.id === grabbedItemId);
      if (item) {
        item.x = cursorX;
        item.y = cursorY;
        items = [...items]; // Trigger reactivity
      }
    }
    
    // Gesture transitions
    if (gesture !== previousGesture) {
      addLog(`${previousGesture} → ${gesture}`);
      
      // GRAB: Closed Fist - find nearest item
      if (gesture === 'Closed_Fist' && grabbedItemId === null) {
        let nearestItem = null;
        let nearestDist = Infinity;
        
        for (const item of items) {
          const dist = Math.sqrt(Math.pow(cursorX - item.x, 2) + Math.pow(cursorY - item.y, 2));
          if (dist < nearestDist && dist < 30) { // Within 30 units
            nearestDist = dist;
            nearestItem = item;
          }
        }
        
        if (nearestItem) {
          grabbedItemId = nearestItem.id;
          nearestItem.grabbed = true;
          items = [...items];
          addLog(`🤜 GRABBED: ${nearestItem.name} (dist: ${nearestDist.toFixed(0)})`);
        } else {
          addLog(`❌ No item nearby (cursor: ${cursorX.toFixed(0)}, ${cursorY.toFixed(0)})`);
        }
      }
      
      // DROP: Open Palm releases
      if (gesture === 'Open_Palm' && grabbedItemId !== null) {
        const item = items.find(i => i.id === grabbedItemId);
        if (item) {
          item.grabbed = false;
          items = [...items];
          addLog(`🖐️ DROPPED: ${item.name} at (${item.x.toFixed(0)}, ${item.y.toFixed(0)})`);
        }
        grabbedItemId = null;
      }
      
      previousGesture = gesture;
    }
  });
  
  function addLog(msg: string) {
    gestureLog = [`${msg}`, ...gestureLog.slice(0, 5)];
  }
  
  function reset() {
    items = [
      { id: 1, name: '📷 Photo', x: 25, y: 50, grabbed: false },
      { id: 2, name: '📄 Doc', x: 50, y: 50, grabbed: false },
      { id: 3, name: '🎵 Music', x: 75, y: 50, grabbed: false },
    ];
    grabbedItemId = null;
    gestureLog = ['Reset!'];
  }
</script>

<div class="grab-test">
  <div class="header">
    <h2>🎯 Grab Test</h2>
    <span class="badge" class:active={grabbedItemId !== null}>
      {$handState.gesture}
    </span>
    <button onclick={reset}>Reset</button>
  </div>
  
  <div class="debug">
    Cursor: ({cursorX.toFixed(0)}, {cursorY.toFixed(0)}) | 
    {grabbedItemId ? `Holding: ${items.find(i => i.id === grabbedItemId)?.name}` : 'Nothing grabbed'}
  </div>
  
  <div class="arena">
    <!-- Cursor -->
    <div class="cursor" style="left: {cursorX}%; top: {cursorY}%;"></div>
    
    <!-- Items -->
    {#each items as item (item.id)}
      <div 
        class="item" 
        class:grabbed={item.grabbed}
        style="left: {item.x}%; top: {item.y}%;"
      >
        {item.name}
      </div>
    {/each}
  </div>
  
  <div class="log">
    {#each gestureLog as log}
      <div>{log}</div>
    {/each}
  </div>
</div>

<style>
  .grab-test {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 600px;
    background: rgba(0, 0, 0, 0.95);
    border: 2px solid #00ff88;
    border-radius: 16px;
    padding: 20px;
    color: white;
    font-family: system-ui;
    z-index: 10000;
  }
  
  .header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }
  
  .header h2 { margin: 0; flex: 1; }
  
  .badge {
    padding: 6px 12px;
    background: #333;
    border-radius: 16px;
    font-size: 14px;
  }
  
  .badge.active {
    background: #00ff88;
    color: black;
  }
  
  button {
    padding: 8px 16px;
    background: #444;
    border: none;
    border-radius: 8px;
    color: white;
    cursor: pointer;
  }
  
  .debug {
    background: #220;
    border: 1px solid #ff0;
    padding: 8px;
    border-radius: 8px;
    font-family: monospace;
    font-size: 12px;
    color: #ff0;
    margin-bottom: 12px;
  }
  
  .arena {
    position: relative;
    height: 300px;
    background: linear-gradient(135deg, #1a1a2e, #16213e);
    border-radius: 12px;
    overflow: hidden;
  }
  
  .cursor {
    position: absolute;
    width: 30px;
    height: 30px;
    background: rgba(255, 255, 255, 0.3);
    border: 3px solid #fff;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    z-index: 100;
  }
  
  .item {
    position: absolute;
    padding: 12px 20px;
    background: linear-gradient(135deg, #667eea, #764ba2);
    border-radius: 12px;
    font-weight: bold;
    transform: translate(-50%, -50%);
    transition: all 0.1s;
    cursor: grab;
  }
  
  .item.grabbed {
    background: linear-gradient(135deg, #00ff88, #00cc66);
    color: black;
    transform: translate(-50%, -50%) scale(1.2);
    box-shadow: 0 0 30px rgba(0, 255, 136, 0.6);
    z-index: 50;
  }
  
  .log {
    margin-top: 12px;
    font-family: monospace;
    font-size: 11px;
    background: #111;
    border-radius: 8px;
    padding: 8px;
    max-height: 80px;
    overflow-y: auto;
    color: #0f8;
  }
</style>
