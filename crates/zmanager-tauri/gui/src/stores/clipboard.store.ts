/**
 * Clipboard Store - Zustand store for file clipboard operations
 *
 * Manages copy/cut/paste state with keybinding support.
 */

import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

// ============================================================================
// Types
// ============================================================================

/** Clipboard operation type */
export type ClipboardOperation = "copy" | "cut";

/** Clipboard state DTO from backend */
interface ClipboardDto {
  paths: string[];
  operation: ClipboardOperation | null;
}

// ============================================================================
// Store State
// ============================================================================

interface ClipboardState {
  /** Paths currently in clipboard */
  paths: string[];
  /** Current operation (copy or cut) */
  operation: ClipboardOperation | null;
  /** Loading state for paste operation */
  isPasting: boolean;
  /** Error message */
  error: string | null;

  // Actions
  copyPaths: (paths: string[]) => Promise<boolean>;
  cutPaths: (paths: string[]) => Promise<boolean>;
  paste: (destination: string) => Promise<number>;
  getClipboard: () => Promise<void>;
  clear: () => Promise<void>;
  hasContent: () => boolean;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  paths: [],
  operation: null,
  isPasting: false,
  error: null,

  copyPaths: async (paths: string[]) => {
    try {
      await invoke("zmanager_clipboard_copy", { paths });
      set({ paths, operation: "copy", error: null });
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    }
  },

  cutPaths: async (paths: string[]) => {
    try {
      await invoke("zmanager_clipboard_cut", { paths });
      set({ paths, operation: "cut", error: null });
      return true;
    } catch (err) {
      set({ error: String(err) });
      return false;
    }
  },

  paste: async (destination: string) => {
    set({ isPasting: true, error: null });
    try {
      const count = await invoke<number>("zmanager_clipboard_paste", { destination });

      // If it was a cut operation, clear local state
      if (get().operation === "cut") {
        set({ paths: [], operation: null });
      }

      set({ isPasting: false });
      return count;
    } catch (err) {
      set({ isPasting: false, error: String(err) });
      return 0;
    }
  },

  getClipboard: async () => {
    try {
      const dto = await invoke<ClipboardDto>("zmanager_clipboard_get");
      set({ paths: dto.paths, operation: dto.operation, error: null });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  clear: async () => {
    try {
      await invoke("zmanager_clipboard_clear");
      set({ paths: [], operation: null, error: null });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  hasContent: () => get().paths.length > 0,
}));
