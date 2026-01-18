/**
 * UI Store
 *
 * Manages UI state like sidebar visibility, modals, and theme.
 */

import { create } from "zustand";

/** Sidebar sections that can be expanded/collapsed */
export type SidebarSection = "favorites" | "drives" | "quickAccess";

/** Pane layout mode */
export type PaneMode = "single" | "dual";

/** UI store state */
interface UIState {
  /** Whether the sidebar is visible */
  sidebarVisible: boolean;
  /** Expanded sidebar sections */
  expandedSections: Set<SidebarSection>;
  /** Whether properties panel is visible */
  propertiesVisible: boolean;
  /** Whether the app is in fullscreen mode */
  isFullscreen: boolean;
  /** Current modal (if any) */
  activeModal: string | null;
  /** Pane layout mode (single or dual pane) */
  paneMode: PaneMode;

  // Actions
  /** Toggle sidebar visibility */
  toggleSidebar: () => void;
  /** Set sidebar visibility */
  setSidebarVisible: (visible: boolean) => void;
  /** Toggle a sidebar section */
  toggleSection: (section: SidebarSection) => void;
  /** Toggle properties panel */
  toggleProperties: () => void;
  /** Set fullscreen state */
  setFullscreen: (fullscreen: boolean) => void;
  /** Open a modal */
  openModal: (modalId: string) => void;
  /** Close the active modal */
  closeModal: () => void;
  /** Toggle pane mode between single and dual */
  togglePaneMode: () => void;
  /** Set pane mode directly */
  setPaneMode: (mode: PaneMode) => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarVisible: true,
  expandedSections: new Set(["favorites", "drives"]),
  propertiesVisible: false,
  isFullscreen: false,
  activeModal: null,
  paneMode: "single", // Default to single pane

  toggleSidebar: () => {
    set((state) => ({ sidebarVisible: !state.sidebarVisible }));
  },

  setSidebarVisible: (visible) => {
    set({ sidebarVisible: visible });
  },

  toggleSection: (section) => {
    set((state) => {
      const newSections = new Set(state.expandedSections);
      if (newSections.has(section)) {
        newSections.delete(section);
      } else {
        newSections.add(section);
      }
      return { expandedSections: newSections };
    });
  },

  toggleProperties: () => {
    set((state) => ({ propertiesVisible: !state.propertiesVisible }));
  },

  setFullscreen: (fullscreen) => {
    set({ isFullscreen: fullscreen });
  },

  openModal: (modalId) => {
    set({ activeModal: modalId });
  },

  closeModal: () => {
    set({ activeModal: null });
  },

  togglePaneMode: () => {
    set((state) => ({
      paneMode: state.paneMode === "single" ? "dual" : "single",
    }));
  },

  setPaneMode: (mode) => {
    set({ paneMode: mode });
  },
}));
