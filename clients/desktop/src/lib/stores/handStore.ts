import { writable, derived } from 'svelte/store';

export type GestureType = 'Closed_Fist' | 'Open_Palm' | 'Pointing_Up' | 'Thumb_Down' | 'Thumb_Up' | 'Victory' | 'ILoveYou' | 'None';

export interface CursorPosition {
  x: number;
  y: number;
}

export interface Landmark3D {
  x: number;
  y: number;
  z: number;
}

export interface HandState {
  isHandDetected: boolean;
  gesture: GestureType;
  cursorPosition: CursorPosition;
  rawPosition: CursorPosition;
  confidence: number;
  landmarks: Landmark3D[] | null; // 21 MediaPipe landmarks
}

const initialState: HandState = {
  isHandDetected: false,
  gesture: 'None',
  cursorPosition: { x: 0.5, y: 0.5 },
  rawPosition: { x: 0.5, y: 0.5 },
  confidence: 0,
  landmarks: null
};

// Main hand state store
export const handState = writable<HandState>(initialState);

// Smoothing factor for Lerp (0 = no smoothing, 1 = instant)
const SMOOTHING_FACTOR = 0.3;

// Lerp function for smooth cursor movement
function lerp(start: number, end: number, factor: number): number {
  return start + (end - start) * factor;
}

// Update hand state with new detection
export function updateHandState(
  detected: boolean,
  gesture: GestureType,
  rawX: number,
  rawY: number,
  confidence: number,
  landmarks: Landmark3D[] | null = null
) {
  handState.update((state) => {
    // Invert X-axis for mirrored webcam (moving right should move cursor right)
    const invertedX = 1 - rawX;
    
    // Apply Lerp smoothing to cursor position
    const smoothedX = lerp(state.cursorPosition.x, invertedX, SMOOTHING_FACTOR);
    const smoothedY = lerp(state.cursorPosition.y, rawY, SMOOTHING_FACTOR);

    return {
      isHandDetected: detected,
      gesture: detected ? gesture : 'None',
      cursorPosition: { x: smoothedX, y: smoothedY },
      rawPosition: { x: invertedX, y: rawY },
      confidence,
      landmarks: detected ? landmarks : null
    };
  });
}

// Clear hand state (hand lost)
export function clearHandState() {
  handState.update((state) => ({
    ...state,
    isHandDetected: false,
    gesture: 'None',
    confidence: 0
  }));
}

// Derived store for grabbing state
export const isGrabbing = derived(handState, ($state) => 
  $state.isHandDetected && $state.gesture === 'Closed_Fist'
);

// Derived store for hovering state
export const isHovering = derived(handState, ($state) => 
  $state.isHandDetected && $state.gesture === 'Open_Palm'
);
