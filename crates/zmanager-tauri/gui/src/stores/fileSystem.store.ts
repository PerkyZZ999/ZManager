/**
 * File System Store
 *
 * Manages navigation state, directory listings, and file entries
 * for both left and right panes.
 */

import { create } from "zustand";
import { getDrives, navigate } from "../lib/tauri";
import type { DirListing, DriveInfo, FilterSpec, SortSpec } from "../types";
import { DEFAULT_FILTER, DEFAULT_SORT } from "../types";

/** Which pane is being referenced */
export type PaneId = "left" | "right";

/** State for a single file pane */
export interface PaneState {
  /** Current directory path */
  path: string;
  /** Directory listing (entries + stats) */
  listing: DirListing | null;
  /** Loading state */
  isLoading: boolean;
  /** Error message if any */
  error: string | null;
  /** Navigation history: back stack */
  historyBack: string[];
  /** Navigation history: forward stack */
  historyForward: string[];
  /** Current sort specification */
  sort: SortSpec;
  /** Current filter specification */
  filter: FilterSpec;
  /** Selected entry indices */
  selectedIndices: Set<number>;
  /** Cursor index (focused entry) */
  cursorIndex: number;
}

/** Global file system store state */
interface FileSystemState {
  /** Left pane state */
  left: PaneState;
  /** Right pane state */
  right: PaneState;
  /** Which pane is active/focused */
  activePane: PaneId;
  /** Available drives */
  drives: DriveInfo[];
  /** Loading drives */
  drivesLoading: boolean;

  // Actions
  /** Set active pane */
  setActivePane: (pane: PaneId) => void;
  /** Navigate to a path in the specified pane */
  navigateTo: (pane: PaneId, path: string) => Promise<void>;
  /** Go back in history */
  goBack: (pane: PaneId) => Promise<void>;
  /** Go forward in history */
  goForward: (pane: PaneId) => Promise<void>;
  /** Go to parent directory */
  goUp: (pane: PaneId) => Promise<void>;
  /** Refresh current directory */
  refresh: (pane: PaneId) => Promise<void>;
  /** Load available drives */
  loadDrives: () => Promise<void>;
  /** Set sort specification */
  setSort: (pane: PaneId, sort: SortSpec) => void;
  /** Set filter specification */
  setFilter: (pane: PaneId, filter: FilterSpec) => void;
  /** Set selected indices */
  setSelection: (pane: PaneId, indices: Set<number>) => void;
  /** Set cursor index */
  setCursor: (pane: PaneId, index: number) => void;
  /** Clear selection */
  clearSelection: (pane: PaneId) => void;
}

/** Create initial pane state */
function createInitialPaneState(defaultPath: string): PaneState {
  return {
    path: defaultPath,
    listing: null,
    isLoading: false,
    error: null,
    historyBack: [],
    historyForward: [],
    sort: DEFAULT_SORT,
    filter: DEFAULT_FILTER,
    selectedIndices: new Set(),
    cursorIndex: 0,
  };
}

/** Get parent path, handling Windows drive roots */
function getParentPath(path: string): string | null {
  // Normalize path
  const normalized = path.replace(/\\/g, "/").replace(/\/$/, "");

  // Check if we're at a drive root (e.g., "C:")
  if (/^[a-zA-Z]:$/.test(normalized)) {
    return null; // At root, no parent
  }

  // Get parent
  const lastSlash = normalized.lastIndexOf("/");
  if (lastSlash === -1) {
    return null;
  }

  const parent = normalized.substring(0, lastSlash);

  // If parent is just a drive letter, return with trailing
  if (/^[a-zA-Z]:$/.test(parent)) {
    return `${parent}\\`;
  }

  return parent || null;
}

export const useFileSystemStore = create<FileSystemState>((set, get) => ({
  left: createInitialPaneState("C:\\"),
  right: createInitialPaneState("D:\\"),
  activePane: "left",
  drives: [],
  drivesLoading: false,

  setActivePane: (pane) => {
    set({ activePane: pane });
  },

  navigateTo: async (pane, path) => {
    const state = get();
    const paneState = state[pane];

    // Set loading state
    set({
      [pane]: {
        ...paneState,
        isLoading: true,
        error: null,
      },
    });

    try {
      const listing = await navigate(path, paneState.sort, paneState.filter);

      // Update history: push current path to back stack, clear forward stack
      const newHistoryBack =
        paneState.path !== path
          ? [...paneState.historyBack, paneState.path]
          : paneState.historyBack;

      set({
        [pane]: {
          ...paneState,
          path,
          listing,
          isLoading: false,
          error: null,
          historyBack: newHistoryBack.slice(-100), // Max 100 entries
          historyForward: [],
          selectedIndices: new Set(),
          cursorIndex: 0,
        },
      });
    } catch (error) {
      set({
        [pane]: {
          ...paneState,
          isLoading: false,
          error: error instanceof Error ? error.message : String(error),
        },
      });
    }
  },

  goBack: async (pane) => {
    const paneState = get()[pane];
    if (paneState.historyBack.length === 0) return;

    const previousPath = paneState.historyBack[paneState.historyBack.length - 1];
    const newHistoryBack = paneState.historyBack.slice(0, -1);
    // Capture the updated historyForward before async operation
    const newHistoryForward = [paneState.path, ...paneState.historyForward];

    set({
      [pane]: {
        ...paneState,
        historyForward: newHistoryForward,
        historyBack: newHistoryBack,
        isLoading: true,
      },
    });

    try {
      const listing = await navigate(previousPath, paneState.sort, paneState.filter);
      set({
        [pane]: {
          ...get()[pane],
          path: previousPath,
          listing,
          isLoading: false,
          error: null,
          selectedIndices: new Set(),
          cursorIndex: 0,
          // Explicitly preserve historyForward to avoid race conditions
          historyForward: newHistoryForward,
        },
      });
    } catch (error) {
      set({
        [pane]: {
          ...get()[pane],
          isLoading: false,
          error: error instanceof Error ? error.message : String(error),
        },
      });
    }
  },

  goForward: async (pane) => {
    const paneState = get()[pane];
    if (paneState.historyForward.length === 0) return;

    const nextPath = paneState.historyForward[0];
    const newHistoryForward = paneState.historyForward.slice(1);
    // Capture the updated historyBack before async operation
    const newHistoryBack = [...paneState.historyBack, paneState.path];

    set({
      [pane]: {
        ...paneState,
        historyBack: newHistoryBack,
        historyForward: newHistoryForward,
        isLoading: true,
      },
    });

    try {
      const listing = await navigate(nextPath, paneState.sort, paneState.filter);
      set({
        [pane]: {
          ...get()[pane],
          path: nextPath,
          listing,
          isLoading: false,
          error: null,
          selectedIndices: new Set(),
          cursorIndex: 0,
          // Explicitly preserve history stacks to avoid race conditions
          historyBack: newHistoryBack,
          historyForward: newHistoryForward,
        },
      });
    } catch (error) {
      set({
        [pane]: {
          ...get()[pane],
          isLoading: false,
          error: error instanceof Error ? error.message : String(error),
        },
      });
    }
  },

  goUp: async (pane) => {
    const paneState = get()[pane];
    const parentPath = getParentPath(paneState.path);
    if (!parentPath) return;

    await get().navigateTo(pane, parentPath);
  },

  refresh: async (pane) => {
    const paneState = get()[pane];
    set({
      [pane]: {
        ...paneState,
        isLoading: true,
        error: null,
      },
    });

    try {
      const listing = await navigate(paneState.path, paneState.sort, paneState.filter);
      set({
        [pane]: {
          ...get()[pane],
          listing,
          isLoading: false,
          error: null,
        },
      });
    } catch (error) {
      set({
        [pane]: {
          ...get()[pane],
          isLoading: false,
          error: error instanceof Error ? error.message : String(error),
        },
      });
    }
  },

  loadDrives: async () => {
    set({ drivesLoading: true });
    try {
      const drives = await getDrives();
      set({ drives, drivesLoading: false });
    } catch (_error) {
      set({ drivesLoading: false });
    }
  },

  setSort: (pane, sort) => {
    const paneState = get()[pane];
    set({
      [pane]: { ...paneState, sort },
    });
    // Refresh to apply new sort
    get().refresh(pane);
  },

  setFilter: (pane, filter) => {
    const paneState = get()[pane];
    set({
      [pane]: { ...paneState, filter },
    });
    // Refresh to apply new filter
    get().refresh(pane);
  },

  setSelection: (pane, indices) => {
    const paneState = get()[pane];
    set({
      [pane]: { ...paneState, selectedIndices: indices },
    });
  },

  setCursor: (pane, index) => {
    const paneState = get()[pane];
    set({
      [pane]: { ...paneState, cursorIndex: index },
    });
  },

  clearSelection: (pane) => {
    const paneState = get()[pane];
    set({
      [pane]: { ...paneState, selectedIndices: new Set() },
    });
  },
}));
