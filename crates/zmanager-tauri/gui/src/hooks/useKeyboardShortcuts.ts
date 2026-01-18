/**
 * Keyboard Shortcuts Hook
 *
 * Handles global keyboard shortcuts for the file manager.
 */

import { useCallback, useEffect } from "react";
import { useToast } from "../components/Toast";
import { useClipboardStore, useFileSystemStore } from "../stores";
import type { EntryMeta } from "../types";

/**
 * Global keyboard shortcuts for the application.
 *
 * - Ctrl+C: Copy selected files
 * - Ctrl+X: Cut selected files
 * - Ctrl+V: Paste files
 * - Delete: Delete selected files
 * - F2: Rename selected file
 * - F5: Refresh current directory (or copy to other pane with Shift)
 * - F6: Move to other pane
 * - Tab: Switch active pane
 * - Escape: Clear selection
 */
export function useKeyboardShortcuts() {
  const { activePane, left, right, refresh, setActivePane, clearSelection } = useFileSystemStore();
  const { copyPaths, cutPaths, paste, hasContent } = useClipboardStore();
  const { success, error, info } = useToast();

  const getCurrentPane = useCallback(() => {
    return activePane === "left" ? left : right;
  }, [activePane, left, right]);

  const getOtherPane = useCallback(() => {
    return activePane === "left" ? right : left;
  }, [activePane, left, right]);

  const getSelectedPaths = useCallback(() => {
    const pane = getCurrentPane();
    if (!pane.listing) return [];

    const entries = pane.listing.entries;
    const paths: string[] = [];

    for (const idx of pane.selectedIndices) {
      const entry = entries[idx] as EntryMeta | undefined;
      if (entry) {
        paths.push(entry.path);
      }
    }
    return paths;
  }, [getCurrentPane]);

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      // Ignore if typing in an input
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
        return;
      }

      const pane = getCurrentPane();
      const otherPane = getOtherPane();

      // Tab: Switch active pane
      if (e.key === "Tab" && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        const newPane = activePane === "left" ? "right" : "left";
        setActivePane(newPane);
        return;
      }

      // Escape: Clear selection
      if (e.key === "Escape") {
        e.preventDefault();
        clearSelection(activePane);
        return;
      }

      // Ctrl+C: Copy
      if (e.ctrlKey && e.key === "c") {
        e.preventDefault();
        const paths = getSelectedPaths();
        if (paths.length > 0) {
          const ok = await copyPaths(paths);
          if (ok) {
            success(`${paths.length} item${paths.length > 1 ? "s" : ""} copied`);
          }
        }
        return;
      }

      // Ctrl+X: Cut
      if (e.ctrlKey && e.key === "x") {
        e.preventDefault();
        const paths = getSelectedPaths();
        if (paths.length > 0) {
          const ok = await cutPaths(paths);
          if (ok) {
            info(`${paths.length} item${paths.length > 1 ? "s" : ""} cut`);
          }
        }
        return;
      }

      // Ctrl+V: Paste
      if (e.ctrlKey && e.key === "v") {
        e.preventDefault();
        if (hasContent()) {
          const count = await paste(pane.path);
          if (count > 0) {
            success(`${count} item${count > 1 ? "s" : ""} pasted`);
            // Refresh directory
            refresh(activePane);
          } else {
            error("Paste failed");
          }
        }
        return;
      }

      // F5: Refresh OR Copy to other pane (with Shift)
      if (e.key === "F5") {
        e.preventDefault();
        if (e.shiftKey) {
          // Shift+F5: Copy to other pane (traditional dual-pane)
          const paths = getSelectedPaths();
          if (paths.length > 0) {
            const ok = await copyPaths(paths);
            if (ok) {
              const count = await paste(otherPane.path);
              if (count > 0) {
                success(`Copied ${count} item${count > 1 ? "s" : ""} to other pane`);
                refresh(activePane === "left" ? "right" : "left");
              }
            }
          } else {
            info("Select files to copy");
          }
        } else {
          // F5: Refresh
          refresh(activePane);
          info("Refreshed");
        }
        return;
      }

      // F6: Move to other pane (traditional dual-pane)
      if (e.key === "F6") {
        e.preventDefault();
        const paths = getSelectedPaths();
        if (paths.length > 0) {
          const ok = await cutPaths(paths);
          if (ok) {
            const count = await paste(otherPane.path);
            if (count > 0) {
              success(`Moved ${count} item${count > 1 ? "s" : ""} to other pane`);
              refresh("left");
              refresh("right");
            }
          }
        } else {
          info("Select files to move");
        }
        return;
      }
    },
    [
      activePane,
      copyPaths,
      cutPaths,
      error,
      getCurrentPane,
      getOtherPane,
      getSelectedPaths,
      hasContent,
      info,
      paste,
      refresh,
      success,
      setActivePane,
      clearSelection,
    ]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}
