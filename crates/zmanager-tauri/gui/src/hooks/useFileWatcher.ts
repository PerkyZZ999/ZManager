/**
 * File Watcher Hook
 *
 * Implements directory refresh when:
 * - Window regains focus
 * - Tab becomes visible after being hidden
 *
 * This is a lightweight alternative to full file watching that
 * catches most real-world scenarios where files change.
 */

import { useEffect, useRef } from "react";
import { useFileSystemStore } from "../stores";

/**
 * Auto-refresh directories when window regains focus or visibility.
 *
 * This is useful when the user switches to Explorer, makes changes,
 * then switches back to ZManager.
 */
export function useFileWatcher() {
  const { left, right, refresh } = useFileSystemStore();
  const lastRefreshRef = useRef<number>(0);
  const MIN_REFRESH_INTERVAL = 2000; // Don't refresh more than once per 2 seconds

  useEffect(() => {
    const doRefresh = () => {
      const now = Date.now();
      if (now - lastRefreshRef.current < MIN_REFRESH_INTERVAL) {
        return; // Debounce rapid focus/visibility changes
      }
      lastRefreshRef.current = now;

      // Refresh both panes if they have content
      if (left.listing) {
        refresh("left");
      }
      if (right.listing) {
        refresh("right");
      }
    };

    // Handle window focus
    const handleFocus = () => {
      doRefresh();
    };

    // Handle visibility change (tab switching)
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        doRefresh();
      }
    };

    window.addEventListener("focus", handleFocus);
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      window.removeEventListener("focus", handleFocus);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [left.listing, right.listing, refresh]);
}
