<script lang="ts">
  import { T, Canvas } from '@threlte/core';
  import { handState, isGrabbing } from '$lib/stores/handStore';
  import { transferState } from '$lib/stores/transferStore';
  import { 
    mediapipeToThree, 
    BONE_CONNECTIONS, 
    getPalmCenter, 
    getIndexTipPosition,
    type Landmark 
  } from '$lib/utils/handLandmarks';
  import { spring } from 'svelte/motion';
  
  // Reactive landmarks from hand store
  $: landmarks = $handState.landmarks as Landmark[] | null;
  $: isHandDetected = $handState.isHandDetected;
  $: grabbing = $isGrabbing;
  $: hasFile = $transferState.isLocalGrab;
  
  // Visual state colors
  $: jointColor = grabbing ? '#00ff88' : '#00d4ff';
  $: boneColor = grabbing ? 'rgba(0, 255, 136, 0.6)' : 'rgba(0, 212, 255, 0.4)';
  $: jointEmissive = grabbing ? 0x00ff88 : 0x00d4ff;
  
  // Smooth opacity transitions
  const opacity = spring(0, { stiffness: 0.1, damping: 0.5 });
  $: opacity.set(isHandDetected ? (grabbing ? 1.0 : 0.6) : 0);
  
  // Calculate 3D positions for all landmarks
  $: positions = landmarks?.map(lm => mediapipeToThree(lm)) ?? [];
  
  // Palm center for file cube
  $: palmCenter = landmarks ? getPalmCenter(landmarks) : [0, 0, 0] as [number, number, number];
  
  // Index tip for cursor ring
  $: indexTip = landmarks ? getIndexTipPosition(landmarks) : [0, 0, 0] as [number, number, number];
  
  // Pulse animation for grab effect
  let pulseScale = 1;
  let pulseInterval: ReturnType<typeof setInterval> | null = null;
  
  $: {
    if (grabbing && !pulseInterval) {
      // Start pulse animation
      pulseScale = 1.5;
      setTimeout(() => pulseScale = 1, 200);
    }
  }
</script>

<div class="cyber-hand-container">
  <Canvas>
    <!-- Camera positioned to view the hand -->
    <T.PerspectiveCamera
      makeDefault
      position={[0, 0, 10]}
      fov={60}
    />
    
    <!-- Ambient light for base visibility -->
    <T.AmbientLight intensity={0.3} />
    
    <!-- Point light for neon glow effect -->
    <T.PointLight position={[0, 5, 10]} intensity={1} color={jointEmissive} />
    
    {#if isHandDetected && positions.length === 21}
      <!-- JOINTS: Spheres at each landmark -->
      {#each positions as pos, i}
        <T.Mesh position={pos}>
          <T.SphereGeometry args={[0.08, 16, 16]} />
          <T.MeshStandardMaterial
            color={jointColor}
            emissive={jointEmissive}
            emissiveIntensity={0.5}
            transparent
            opacity={$opacity}
          />
        </T.Mesh>
      {/each}
      
      <!-- BONES: Lines connecting landmarks -->
      {#each BONE_CONNECTIONS as [startIdx, endIdx]}
        {@const start = positions[startIdx]}
        {@const end = positions[endIdx]}
        <T.Line>
          <T.BufferGeometry>
            <T.BufferAttribute
              attach="attributes-position"
              args={[new Float32Array([...start, ...end]), 3]}
            />
          </T.BufferGeometry>
          <T.LineBasicMaterial
            color={jointColor}
            transparent
            opacity={$opacity * 0.6}
            linewidth={2}
          />
        </T.Line>
      {/each}
      
      <!-- CURSOR RING: At index finger tip -->
      <T.Mesh position={indexTip} rotation={[Math.PI / 2, 0, 0]}>
        <T.TorusGeometry args={[0.15, 0.03, 16, 32]} />
        <T.MeshStandardMaterial
          color={jointColor}
          emissive={jointEmissive}
          emissiveIntensity={0.8}
          transparent
          opacity={$opacity}
        />
      </T.Mesh>
      
      <!-- PULSE EFFECT: At wrist when grabbing -->
      {#if grabbing}
        <T.Mesh position={positions[0]} scale={[pulseScale, pulseScale, pulseScale]}>
          <T.RingGeometry args={[0.2, 0.4, 32]} />
          <T.MeshBasicMaterial
            color="#00ff88"
            transparent
            opacity={0.5 / pulseScale}
            side={2}
          />
        </T.Mesh>
      {/if}
      
      <!-- FILE CUBE: Floating at palm when holding -->
      {#if hasFile}
        <T.Mesh position={palmCenter}>
          <T.BoxGeometry args={[0.3, 0.3, 0.3]} />
          <T.MeshStandardMaterial
            color="#ffaa00"
            emissive={0xffaa00}
            emissiveIntensity={0.3}
            transparent
            opacity={0.8}
          />
        </T.Mesh>
      {/if}
    {/if}
  </Canvas>
</div>

<style>
  .cyber-hand-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 9999;
  }
  
  :global(.cyber-hand-container canvas) {
    background: transparent !important;
  }
</style>
