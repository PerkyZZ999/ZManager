/**
 * ZManager GUI Application
 *
 * Main application component that renders the complete file manager UI.
 */

import { useCallback } from "react";
import {
  ContextMenuProvider,
  DialogProvider,
  DndProvider,
  FilePane,
  ResizablePanes,
  Sidebar,
  StatusBar,
  TitleBar,
  ToastProvider,
  useToast,
} from "./components";
import { useFileWatcher, useKeyboardShortcuts } from "./hooks";
import { type PaneId, useClipboardStore, useFileSystemStore, useUIStore } from "./stores";
import type { EntryMeta } from "./types";

/** Inner shell component that uses hooks requiring context */
function AppShell() {
  // Initialize global keyboard shortcuts
  useKeyboardShortcuts();

  // Auto-refresh on focus/visibility change
  useFileWatcher();

  const { cutPaths, paste } = useClipboardStore();
  const { refresh } = useFileSystemStore();
  const { paneMode } = useUIStore();
  const toast = useToast();

  // Handle internal drag-and-drop between panes
  const handleInternalMove = useCallback(
    async (
      entries: EntryMeta[],
      sourcePaneId: PaneId,
      targetPaneId: PaneId,
      targetPath: string
    ) => {
      if (entries.length === 0) return;

      const paths = entries.map((e) => e.path);

      // Use cut+paste for move operation
      const ok = await cutPaths(paths);
      if (ok) {
        const count = await paste(targetPath);
        if (count > 0) {
          toast.success(`Moved ${count} item${count > 1 ? "s" : ""}`);
          // Refresh both panes
          refresh(sourcePaneId);
          refresh(targetPaneId);
        }
      }
    },
    [cutPaths, paste, refresh, toast]
  );

  // Handle external file drop from OS
  const handleFileDrop = useCallback(
    async (files: string[], _targetPane: PaneId, targetPath: string) => {
      if (files.length === 0) return;

      // Copy external files to target
      const { copyPaths, paste: pasteFiles } = useClipboardStore.getState();
      const { refresh: refreshPane } = useFileSystemStore.getState();

      const ok = await copyPaths(files);
      if (ok) {
        const count = await pasteFiles(targetPath);
        if (count > 0) {
          toast.success(`Copied ${count} file${count > 1 ? "s" : ""} from Explorer`);
          refreshPane("left");
          refreshPane("right");
        }
      }
    },
    [toast]
  );

  return (
    <DndProvider onInternalMove={handleInternalMove} onFileDrop={handleFileDrop}>
      <div className="flex h-screen flex-col bg-bg-primary text-text-primary">
        {/* Custom titlebar */}
        <TitleBar />

        {/* Main content area */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <Sidebar />

          {/* File pane view - single or dual */}
          <main className="flex flex-1 overflow-hidden">
            {paneMode === "single" ? (
              <FilePane paneId="left" />
            ) : (
              <ResizablePanes
                leftPane={<FilePane paneId="left" />}
                rightPane={<FilePane paneId="right" />}
              />
            )}
          </main>
        </div>

        {/* Status bar */}
        <StatusBar />
      </div>
    </DndProvider>
  );
}

function App() {
  return (
    <ToastProvider>
      <DialogProvider>
        <ContextMenuProvider>
          <AppShell />
        </ContextMenuProvider>
      </DialogProvider>
    </ToastProvider>
  );
}

export default App;
