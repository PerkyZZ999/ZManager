/**
 * Custom Windows-style TitleBar component.
 *
 * Features:
 * - Draggable region for window movement
 * - App branding (icon + name)
 * - Windows-style minimize/maximize/close buttons
 */

import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import { useUIStore } from "../stores";

/** Check if we're running inside Tauri */
function isTauriEnv(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Safely get current window, returns null if not in Tauri */
function safeGetCurrentWindow(): Window | null {
  if (!isTauriEnv()) return null;
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

/** Window control button icons (SVG paths) */
const ICONS = {
  minimize: "M4 11h16v2H4z",
  maximize: "M4 4h16v16H4V4zm2 2v12h12V6H6z",
  restore: "M4 8h12v12H4V8zm2 2v8h8v-8H6zM8 4h12v12h-2V6H8V4z",
  close:
    "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z",
  singlePane: "M3 3h18v18H3V3zm2 2v14h14V5H5z",
  dualPane: "M3 3h18v18H3V3zm2 2v14h6V5H5zm8 0v14h6V5h-6z",
};

/** Window control button component */
function WindowButton({
  icon,
  onClick,
  isClose = false,
  label,
}: {
  icon: string;
  onClick: () => void;
  isClose?: boolean;
  label: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      className={`flex h-8 w-12 items-center justify-center transition-colors ${
        isClose ? "hover:bg-red-600 hover:text-white" : "hover:bg-white/10"
      }`}
    >
      <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path d={icon} />
      </svg>
    </button>
  );
}

export function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const { paneMode, togglePaneMode } = useUIStore();

  const handleMinimize = useCallback(async () => {
    const win = safeGetCurrentWindow();
    if (win) await win.minimize();
  }, []);

  const handleMaximize = useCallback(async () => {
    const win = safeGetCurrentWindow();
    if (!win) return;
    const maximized = await win.isMaximized();
    if (maximized) {
      await win.unmaximize();
      setIsMaximized(false);
    } else {
      await win.maximize();
      setIsMaximized(true);
    }
  }, []);

  const handleClose = useCallback(async () => {
    const win = safeGetCurrentWindow();
    if (win) await win.close();
  }, []);

  // Check initial maximized state
  useEffect(() => {
    const win = safeGetCurrentWindow();
    if (win) {
      win
        .isMaximized()
        .then(setIsMaximized)
        .catch(() => {});
    }
  }, []);

  return (
    <header className="flex h-9 select-none items-center justify-between bg-zinc-900 text-zinc-100">
      {/* Draggable region with app branding */}
      <div data-tauri-drag-region className="flex h-full flex-1 items-center gap-2 px-3">
        {/* App icon - TODO: Replace with actual icon */}
        <div className="flex h-5 w-5 items-center justify-center rounded bg-primary/20 text-primary">
          <span className="font-bold text-xs">Z</span>
        </div>
        <span className="font-medium text-sm">ZManager</span>
      </div>

      {/* Pane mode toggle */}
      <button
        type="button"
        onClick={togglePaneMode}
        aria-label={paneMode === "single" ? "Switch to dual pane" : "Switch to single pane"}
        title={paneMode === "single" ? "Switch to dual pane" : "Switch to single pane"}
        className="flex h-8 items-center gap-1.5 rounded-sm px-2.5 text-zinc-300 transition-colors hover:bg-white/10 hover:text-white"
      >
        <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path d={paneMode === "single" ? ICONS.dualPane : ICONS.singlePane} />
        </svg>
        <span className="text-xs">{paneMode === "single" ? "Dual" : "Single"}</span>
      </button>

      {/* Window controls */}
      <div className="flex h-full">
        <WindowButton icon={ICONS.minimize} onClick={handleMinimize} label="Minimize" />
        <WindowButton
          icon={isMaximized ? ICONS.restore : ICONS.maximize}
          onClick={handleMaximize}
          label={isMaximized ? "Restore" : "Maximize"}
        />
        <WindowButton icon={ICONS.close} onClick={handleClose} isClose label="Close" />
      </div>
    </header>
  );
}
