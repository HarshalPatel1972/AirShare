import { writable, derived } from 'svelte/store';

export interface StagedFile {
  id: string;
  path: string;
  name: string;
  size: number;
  type: string;
  extension: string;
  // 3D position for DataCube
  position: { x: number; y: number; z: number };
  // State
  isGrabbed: boolean;
  isTransferring: boolean;
}

// Staged files waiting to be transferred
const stagedFilesStore = writable<StagedFile[]>([]);

// Currently grabbed file (attached to hand)
const grabbedFileStore = writable<StagedFile | null>(null);

// File type to color mapping
export const FILE_COLORS: Record<string, string> = {
  'image': '#ff6b9d',    // Pink
  'video': '#a855f7',    // Purple
  'audio': '#22c55e',    // Green
  'document': '#3b82f6', // Blue
  'archive': '#f59e0b',  // Orange
  'default': '#00d4ff',  // Cyan
};

export function getFileColor(extension: string): string {
  const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'];
  const videoExts = ['mp4', 'mov', 'avi', 'mkv', 'webm'];
  const audioExts = ['mp3', 'wav', 'ogg', 'flac', 'm4a'];
  const docExts = ['pdf', 'doc', 'docx', 'txt', 'xls', 'xlsx', 'ppt', 'pptx'];
  const archiveExts = ['zip', 'rar', '7z', 'tar', 'gz'];
  
  const ext = extension.toLowerCase();
  if (imageExts.includes(ext)) return FILE_COLORS.image;
  if (videoExts.includes(ext)) return FILE_COLORS.video;
  if (audioExts.includes(ext)) return FILE_COLORS.audio;
  if (docExts.includes(ext)) return FILE_COLORS.document;
  if (archiveExts.includes(ext)) return FILE_COLORS.archive;
  return FILE_COLORS.default;
}

// Generate unique ID
function generateId(): string {
  return Math.random().toString(36).substring(2, 9);
}

// Add a file to staging
export function stageFile(path: string, name: string, size: number): StagedFile {
  const extension = name.split('.').pop() || '';
  const type = extension;
  
  // Position new cubes in a row
  let xOffset = 0;
  stagedFilesStore.update(files => {
    xOffset = files.length * 1.5;
    return files;
  });
  
  const file: StagedFile = {
    id: generateId(),
    path,
    name,
    size,
    type,
    extension,
    position: { x: -3 + xOffset, y: 0, z: 0 },
    isGrabbed: false,
    isTransferring: false,
  };
  
  stagedFilesStore.update(files => [...files, file]);
  console.log(`[FileStore] Staged: ${name}`);
  return file;
}

// Grab a file (attach to hand)
export function grabFile(fileId: string): void {
  stagedFilesStore.update(files => {
    const file = files.find(f => f.id === fileId);
    if (file) {
      file.isGrabbed = true;
      grabbedFileStore.set(file);
    }
    return files;
  });
}

// Release grabbed file
export function releaseFile(): void {
  const currentGrabbed = grabbedFileStore;
  grabbedFileStore.set(null);
  stagedFilesStore.update(files => {
    return files.map(f => ({ ...f, isGrabbed: false }));
  });
}

// Remove file from staging (after transfer)
export function unstageFile(fileId: string): void {
  stagedFilesStore.update(files => files.filter(f => f.id !== fileId));
}

// Clear all staged files
export function clearStaged(): void {
  stagedFilesStore.set([]);
  grabbedFileStore.set(null);
}

// Exports
export const stagedFiles = { subscribe: stagedFilesStore.subscribe };
export const grabbedFile = { subscribe: grabbedFileStore.subscribe };

// Derived: count of staged files
export const stagedCount = derived(stagedFilesStore, $files => $files.length);
