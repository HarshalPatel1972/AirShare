// Utility functions for converting MediaPipe landmarks to Three.js coordinates

import type * as THREE from 'three';

// MediaPipe hand landmark indices
export const LANDMARK_INDICES = {
  WRIST: 0,
  THUMB_CMC: 1,
  THUMB_MCP: 2,
  THUMB_IP: 3,
  THUMB_TIP: 4,
  INDEX_MCP: 5,
  INDEX_PIP: 6,
  INDEX_DIP: 7,
  INDEX_TIP: 8,
  MIDDLE_MCP: 9,
  MIDDLE_PIP: 10,
  MIDDLE_DIP: 11,
  MIDDLE_TIP: 12,
  RING_MCP: 13,
  RING_PIP: 14,
  RING_DIP: 15,
  RING_TIP: 16,
  PINKY_MCP: 17,
  PINKY_PIP: 18,
  PINKY_DIP: 19,
  PINKY_TIP: 20
} as const;

// Bone connections (pairs of landmark indices)
export const BONE_CONNECTIONS: [number, number][] = [
  // Thumb
  [0, 1], [1, 2], [2, 3], [3, 4],
  // Index
  [0, 5], [5, 6], [6, 7], [7, 8],
  // Middle
  [0, 9], [9, 10], [10, 11], [11, 12],
  // Ring
  [0, 13], [13, 14], [14, 15], [15, 16],
  // Pinky
  [0, 17], [17, 18], [18, 19], [19, 20],
  // Palm connections
  [5, 9], [9, 13], [13, 17]
];

export interface Landmark {
  x: number;
  y: number;
  z: number;
}

/**
 * Convert MediaPipe normalized coordinates (0-1) to Three.js world coordinates
 * MediaPipe: (0,0) top-left, (1,1) bottom-right
 * Three.js: Origin at center, Y up
 */
export function mediapipeToThree(
  landmark: Landmark,
  screenWidth: number = 16,  // World units for screen width
  screenHeight: number = 9   // World units for screen height (16:9 aspect)
): [number, number, number] {
  // Invert X for mirror effect, center coordinates
  const x = (1 - landmark.x - 0.5) * screenWidth;
  
  // Invert Y (MediaPipe Y increases downward, Three.js Y increases upward)
  const y = (0.5 - landmark.y) * screenHeight;
  
  // Scale Z for depth (MediaPipe z is relative to wrist, typically small values)
  // Multiply by factor to make depth more visible
  const z = -landmark.z * 5;
  
  return [x, y, z];
}

/**
 * Calculate the center of the palm (average of key landmarks)
 */
export function getPalmCenter(landmarks: Landmark[]): [number, number, number] {
  const palmIndices = [0, 5, 9, 13, 17]; // Wrist + MCP joints
  let x = 0, y = 0, z = 0;
  
  for (const idx of palmIndices) {
    const pos = mediapipeToThree(landmarks[idx]);
    x += pos[0];
    y += pos[1];
    z += pos[2];
  }
  
  return [x / palmIndices.length, y / palmIndices.length, z / palmIndices.length];
}

/**
 * Get the position of the index finger tip (used for cursor)
 */
export function getIndexTipPosition(landmarks: Landmark[]): [number, number, number] {
  return mediapipeToThree(landmarks[LANDMARK_INDICES.INDEX_TIP]);
}
