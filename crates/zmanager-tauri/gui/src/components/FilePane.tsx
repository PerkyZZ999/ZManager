/**
 * FilePane component
 *
 * Displays a single file list pane with:
 * - Header with navigation controls and address bar
 * - Virtualized file list with sorting
 * - Loading, error, and empty states
 * - Keyboard navigation support
 * - Context menu for file operations
 * - Keyboard shortcuts (Delete, F2, Ctrl+Shift+N, Ctrl+C/X/V, Enter)
 * - Drag and drop support (internal and external)
 * - Clipboard operations (Copy, Cut, Paste)
 */

import { useDroppable } from "@dnd-kit/core";
import clsx from "clsx";
import { useCallback, useEffect, useState } from "react";
import { createFile, createFolder, deleteEntries, openFile, renameEntry } from "../lib/tauri";
import { type PaneId, useClipboardStore, useFileSystemStore } from "../stores";
import type { EntryMeta, SortField } from "../types";
import { AddressBar } from "./AddressBar";
import { type MenuEntry, useContextMenu } from "./ContextMenu";
import { useDialog } from "./Dialogs";
import { type DropZoneData, useDnd } from "./DndProvider";
import { NewButton } from "./NewButton";
import { PropertiesPanel } from "./PropertiesPanel";
import { SvgIcon } from "./SvgIcon";
import { useToast } from "./Toast";
import { VirtualizedFileList } from "./VirtualizedFileList";

// ============================================================================
// Search Input Component
// ============================================================================

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

function SearchInput({ value, onChange, placeholder = "Search..." }: SearchInputProps) {
  return (
    <div className="relative">
      <SvgIcon
        name="ic_search"
        size={14}
        alt="Search"
        className="pointer-events-none absolute top-1/2 left-2 -translate-y-1/2 opacity-50"
      />
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-32 rounded bg-zinc-900 py-1 pr-2 pl-7 text-sm text-white placeholder-zinc-500 outline-none transition-all focus:w-48 focus:ring-1 focus:ring-primary/50"
        spellCheck={false}
      />
    </div>
  );
}

// ============================================================================
// Navigation Header Component
// ============================================================================

interface PaneHeaderProps {
  paneId: PaneId;
  path: string;
  canGoBack: boolean;
  canGoForward: boolean;
  searchQuery: string;
  onBack: () => void;
  onForward: () => void;
  onUp: () => void;
  onRefresh: () => void;
  onNavigate: (path: string) => void;
  onSearchChange: (query: string) => void;
  onNewFolder: () => void;
  onNewFile: (name: string) => void;
}

function PaneHeader({
  paneId,
  path,
  canGoBack,
  canGoForward,
  searchQuery,
  onBack,
  onForward,
  onUp,
  onRefresh,
  onNavigate,
  onSearchChange,
  onNewFolder,
  onNewFile,
}: PaneHeaderProps) {
  return (
    <div className="flex items-center gap-1 border-zinc-700 border-b bg-zinc-800 px-2 py-1.5">
      {/* Navigation buttons */}
      <button
        type="button"
        onClick={onBack}
        disabled={!canGoBack}
        className="rounded p-1.5 transition-colors hover:bg-white/10 disabled:opacity-30"
        aria-label="Go back"
        title="Go back (Alt+Left)"
      >
        <SvgIcon name="ic_chevron_left" size={14} />
      </button>
      <button
        type="button"
        onClick={onForward}
        disabled={!canGoForward}
        className="rounded p-1.5 transition-colors hover:bg-white/10 disabled:opacity-30"
        aria-label="Go forward"
        title="Go forward (Alt+Right)"
      >
        <SvgIcon name="ic_chevron_right" size={14} />
      </button>
      <button
        type="button"
        onClick={onUp}
        className="rounded p-1.5 transition-colors hover:bg-white/10"
        aria-label="Go up"
        title="Go to parent folder (Backspace)"
      >
        <SvgIcon name="ic_chevron_up" size={14} />
      </button>
      <button
        type="button"
        onClick={onRefresh}
        className="rounded p-1.5 transition-colors hover:bg-white/10"
        aria-label="Refresh"
        title="Refresh (F5)"
      >
        <SvgIcon name="ic_arrow_clockwise" size={14} />
      </button>

      {/* New button dropdown */}
      <NewButton onNewFolder={onNewFolder} onNewFile={onNewFile} />

      {/* Address bar */}
      <AddressBar path={path} onNavigate={onNavigate} className="ml-1" />

      {/* Search input */}
      <SearchInput value={searchQuery} onChange={onSearchChange} />

      {/* Pane indicator */}
      <span className="ml-1 text-xs text-zinc-500 uppercase tracking-wider">{paneId}</span>
    </div>
  );
}

// ============================================================================
// Main FilePane Component
// ============================================================================

export function FilePane({ paneId }: { paneId: PaneId }) {
  const {
    [paneId]: paneState,
    activePane,
    setActivePane,
    navigateTo,
    goBack,
    goForward,
    goUp,
    refresh,
    setSelection,
    setCursor,
    setSort,
    setFilter,
  } = useFileSystemStore();

  const {
    copyPaths,
    cutPaths,
    paste,
    hasContent: hasClipboardContent,
    operation: clipboardOperation,
  } = useClipboardStore();

  const dialog = useDialog();
  const contextMenu = useContextMenu();
  const toast = useToast();
  const { overPaneId, isExternalDrop } = useDnd();
  const [propertiesEntries, setPropertiesEntries] = useState<EntryMeta[]>([]);

  const {
    path,
    listing,
    isLoading,
    error,
    historyBack,
    historyForward,
    cursorIndex,
    selectedIndices,
    sort,
    filter,
  } = paneState;

  const isActive = activePane === paneId;
  const searchQuery = filter.pattern ?? "";

  // Setup drop zone for this pane
  const dropZoneData: DropZoneData = { type: "pane", paneId, path };
  const { setNodeRef: setDropRef, isOver } = useDroppable({
    id: `pane-${paneId}`,
    data: dropZoneData,
  });

  // Determine if this pane should show drop highlight
  const showDropHighlight = isOver || (isExternalDrop && overPaneId === paneId);

  // Load initial directory on mount - only run once
  useEffect(() => {
    // Only load on mount if we don't already have a listing
    // This prevents overwriting history when path changes via goBack/goForward
    if (!listing) {
      navigateTo(paneId, path);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [paneId]); // Only run once on mount, not on path change

  // Handle entry selection
  const handleSelect = useCallback(
    (index: number, _entry: EntryMeta, event: React.MouseEvent | React.KeyboardEvent) => {
      setActivePane(paneId);

      // Handle multi-select with Ctrl/Shift
      if ("ctrlKey" in event && event.ctrlKey) {
        // Toggle selection
        const newSelection = new Set(selectedIndices);
        if (newSelection.has(index)) {
          newSelection.delete(index);
        } else {
          newSelection.add(index);
        }
        setSelection(paneId, newSelection);
        setCursor(paneId, index);
      } else if ("shiftKey" in event && event.shiftKey && listing) {
        // Range selection
        const start = Math.min(cursorIndex, index);
        const end = Math.max(cursorIndex, index);
        const newSelection = new Set<number>();
        for (let i = start; i <= end; i++) {
          newSelection.add(i);
        }
        setSelection(paneId, newSelection);
        setCursor(paneId, index);
      } else {
        // Single selection
        setCursor(paneId, index);
        setSelection(paneId, new Set([index]));
      }
    },
    [paneId, setActivePane, setCursor, setSelection, selectedIndices, cursorIndex, listing]
  );

  // Handle double-click / Enter to navigate or open
  const handleDoubleClick = useCallback(
    async (entry: EntryMeta) => {
      if (entry.kind === "directory" || entry.kind === "junction") {
        navigateTo(paneId, entry.path);
      } else {
        // Open file with default application
        try {
          await openFile(entry.path);
        } catch (_err) {
          // Error is logged in development
        }
      }
    },
    [paneId, navigateTo]
  );

  // Get selected entries
  const getSelectedEntries = useCallback((): EntryMeta[] => {
    if (!listing) return [];
    return Array.from(selectedIndices)
      .map((i) => listing.entries[i])
      .filter((e): e is EntryMeta => e !== undefined);
  }, [listing, selectedIndices]);

  // Delete selected entries
  const handleDelete = useCallback(async () => {
    const selected = getSelectedEntries();
    if (selected.length === 0) return;

    const confirmed = await dialog.showConfirm({
      title: "Delete",
      message:
        selected.length === 1
          ? `Move "${selected[0].name}" to Recycle Bin?`
          : `Move ${selected.length} items to Recycle Bin?`,
      confirmLabel: "Delete",
      danger: true,
    });

    if (!confirmed) return;

    try {
      const paths = selected.map((e) => e.path);
      const result = await deleteEntries(paths);
      if (result.failed > 0) {
        toast.warning(
          `Deleted ${result.deleted} items`,
          `${result.failed} items failed: ${result.errors[0] ?? "Unknown error"}`
        );
      } else {
        toast.success(`Deleted ${result.deleted} items`);
      }
      refresh(paneId);
    } catch (err) {
      toast.error("Delete failed", err instanceof Error ? err.message : "Unknown error");
    }
  }, [getSelectedEntries, dialog, refresh, paneId, toast]);

  // Rename entry
  const handleRename = useCallback(async () => {
    const selected = getSelectedEntries();
    if (selected.length !== 1) return;

    const entry = selected[0];
    const newName = await dialog.showRename({
      currentName: entry.name,
      isDirectory: entry.kind === "directory",
    });

    if (!newName) return;

    try {
      await renameEntry(entry.path, newName);
      toast.success(`Renamed to "${newName}"`);
      refresh(paneId);
    } catch (err) {
      toast.error("Rename failed", err instanceof Error ? err.message : "Unknown error");
    }
  }, [getSelectedEntries, dialog, refresh, paneId, toast]);

  // Create new folder
  const handleNewFolder = useCallback(async () => {
    const folderName = await dialog.showNewFolder({});

    if (!folderName) return;

    try {
      await createFolder(path, folderName);
      toast.success(`Created folder "${folderName}"`);
      refresh(paneId);
    } catch (err) {
      toast.error("Failed to create folder", err instanceof Error ? err.message : "Unknown error");
    }
  }, [dialog, path, refresh, paneId, toast]);

  // Show properties panel
  const handleShowProperties = useCallback(() => {
    const selected = getSelectedEntries();
    if (selected.length > 0) {
      setPropertiesEntries(selected);
    }
  }, [getSelectedEntries]);

  // Copy selected entries to clipboard
  const handleCopy = useCallback(async () => {
    const selected = getSelectedEntries();
    if (selected.length === 0) return;

    const paths = selected.map((e) => e.path);
    const success = await copyPaths(paths);
    if (success) {
      toast.success(
        selected.length === 1 ? `Copied "${selected[0].name}"` : `Copied ${selected.length} items`
      );
    }
  }, [getSelectedEntries, copyPaths, toast]);

  // Cut selected entries to clipboard
  const handleCut = useCallback(async () => {
    const selected = getSelectedEntries();
    if (selected.length === 0) return;

    const paths = selected.map((e) => e.path);
    const success = await cutPaths(paths);
    if (success) {
      toast.success(
        selected.length === 1 ? `Cut "${selected[0].name}"` : `Cut ${selected.length} items`
      );
    }
  }, [getSelectedEntries, cutPaths, toast]);

  // Paste from clipboard
  const handlePaste = useCallback(async () => {
    if (!hasClipboardContent()) return;

    try {
      const count = await paste(path);
      if (count > 0) {
        toast.success(
          clipboardOperation === "cut"
            ? `Moved ${count} item${count > 1 ? "s" : ""}`
            : `Pasted ${count} item${count > 1 ? "s" : ""}`
        );
        refresh(paneId);
      }
    } catch (err) {
      toast.error("Paste failed", err instanceof Error ? err.message : "Unknown error");
    }
  }, [hasClipboardContent, paste, path, clipboardOperation, toast, refresh, paneId]);

  // Create new text file
  const handleNewFile = useCallback(async () => {
    const fileName = await dialog.showInput({
      title: "New Text Document",
      label: "File name:",
      defaultValue: "New Text Document.txt",
      placeholder: "Enter file name",
    });

    if (!fileName) return;

    try {
      await createFile(path, fileName);
      toast.success(`Created file "${fileName}"`);
      refresh(paneId);
    } catch (err) {
      toast.error("Failed to create file", err instanceof Error ? err.message : "Unknown error");
    }
  }, [dialog, path, refresh, paneId, toast]);

  // Create file directly with a given name (used by NewButton dropdown)
  const handleCreateFile = useCallback(
    async (fileName: string) => {
      try {
        await createFile(path, fileName);
        toast.success(`Created file "${fileName}"`);
        refresh(paneId);
      } catch (err) {
        toast.error("Failed to create file", err instanceof Error ? err.message : "Unknown error");
      }
    },
    [path, refresh, paneId, toast]
  );

  // Context menu handler
  const handleContextMenu = useCallback(
    (e: React.MouseEvent, entry?: EntryMeta) => {
      e.preventDefault();
      setActivePane(paneId);

      // If right-clicking on an unselected entry, select it
      if (entry && listing) {
        const entryIndex = listing.entries.findIndex((e) => e.path === entry.path);
        if (entryIndex >= 0 && !selectedIndices.has(entryIndex)) {
          setCursor(paneId, entryIndex);
          setSelection(paneId, new Set([entryIndex]));
        }
      }

      const selected = entry ? getSelectedEntries() : [];
      const hasSelection = selected.length > 0;
      const singleSelection = selected.length === 1;
      const isDirectory =
        singleSelection && (selected[0].kind === "directory" || selected[0].kind === "junction");
      const canPaste = hasClipboardContent();

      const menuItems: MenuEntry[] = [];

      if (hasSelection) {
        // File/Folder context menu
        menuItems.push({
          id: "open",
          label: isDirectory ? "Open" : "Open",
          icon: "ic_open",
          shortcut: "Enter",
          onClick: () => {
            if (singleSelection) {
              handleDoubleClick(selected[0]);
            }
          },
        });

        menuItems.push({ separator: true });

        // Clipboard operations
        menuItems.push({
          id: "cut",
          label: "Cut",
          icon: "ic_cut",
          shortcut: "Ctrl+X",
          onClick: handleCut,
        });

        menuItems.push({
          id: "copy",
          label: "Copy",
          icon: "ic_copy",
          shortcut: "Ctrl+C",
          onClick: handleCopy,
        });

        menuItems.push({ separator: true });

        menuItems.push({
          id: "rename",
          label: "Rename",
          icon: "ic_rename",
          shortcut: "F2",
          disabled: !singleSelection,
          onClick: handleRename,
        });

        menuItems.push({
          id: "delete",
          label: "Delete",
          icon: "ic_delete",
          shortcut: "Del",
          danger: true,
          onClick: handleDelete,
        });

        menuItems.push({ separator: true });

        menuItems.push({
          id: "properties",
          label: "Properties",
          icon: "ic_info",
          shortcut: "Alt+Enter",
          onClick: handleShowProperties,
        });
      } else {
        // Background context menu (no selection)
        // New submenu items
        menuItems.push({
          id: "new-folder",
          label: "New Folder",
          icon: "ic_folder_add",
          shortcut: "Ctrl+Shift+N",
          onClick: handleNewFolder,
        });

        menuItems.push({
          id: "new-file",
          label: "New Text Document",
          icon: "ic_document_add",
          onClick: handleNewFile,
        });

        menuItems.push({ separator: true });

        // Paste option (only if clipboard has content)
        menuItems.push({
          id: "paste",
          label: "Paste",
          icon: "ic_clipboard_paste",
          shortcut: "Ctrl+V",
          disabled: !canPaste,
          onClick: handlePaste,
        });

        menuItems.push({ separator: true });

        menuItems.push({
          id: "refresh",
          label: "Refresh",
          icon: "ic_arrow_clockwise",
          shortcut: "F5",
          onClick: () => refresh(paneId),
        });
      }

      contextMenu.show(e.clientX, e.clientY, menuItems);
    },
    [
      paneId,
      setActivePane,
      listing,
      selectedIndices,
      setCursor,
      setSelection,
      getSelectedEntries,
      hasClipboardContent,
      handleDoubleClick,
      handleCopy,
      handleCut,
      handlePaste,
      handleRename,
      handleDelete,
      handleShowProperties,
      handleNewFolder,
      handleNewFile,
      refresh,
      contextMenu,
    ]
  );

  // Handle sort field change
  const handleSortChange = useCallback(
    (field: SortField) => {
      const newOrder =
        sort.field === field
          ? sort.order === "ascending"
            ? "descending"
            : "ascending"
          : "ascending";

      setSort(paneId, { ...sort, field, order: newOrder });
    },
    [paneId, sort, setSort]
  );

  // Handle search query change
  const handleSearchChange = useCallback(
    (query: string) => {
      setFilter(paneId, { ...filter, pattern: query || null });
    },
    [paneId, filter, setFilter]
  );

  // Handle keyboard navigation at pane level
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "Backspace":
          e.preventDefault();
          goUp(paneId);
          break;
        case "Escape":
          e.preventDefault();
          setSelection(paneId, new Set());
          break;
        case "Tab":
          // Let global handler manage pane switching
          break;
        case "F5":
          e.preventDefault();
          refresh(paneId);
          break;
        case "F2":
          e.preventDefault();
          handleRename();
          break;
        case "Delete":
          e.preventDefault();
          handleDelete();
          break;
        case "Enter":
          if (e.altKey) {
            e.preventDefault();
            handleShowProperties();
          } else {
            // Open selected entry
            const selected = getSelectedEntries();
            if (selected.length === 1) {
              e.preventDefault();
              handleDoubleClick(selected[0]);
            }
          }
          break;
        case "N":
        case "n":
          if (e.ctrlKey && e.shiftKey) {
            e.preventDefault();
            handleNewFolder();
          }
          break;
        case "c":
        case "C":
          if (e.ctrlKey && !e.shiftKey && !e.altKey) {
            e.preventDefault();
            handleCopy();
          }
          break;
        case "x":
        case "X":
          if (e.ctrlKey && !e.shiftKey && !e.altKey) {
            e.preventDefault();
            handleCut();
          }
          break;
        case "v":
        case "V":
          if (e.ctrlKey && !e.shiftKey && !e.altKey) {
            e.preventDefault();
            handlePaste();
          }
          break;
        case "a":
          if (e.ctrlKey && listing) {
            e.preventDefault();
            // Select all
            const allIndices = new Set<number>();
            for (let i = 0; i < listing.entries.length; i++) {
              allIndices.add(i);
            }
            setSelection(paneId, allIndices);
          }
          break;
      }

      // Alt+Arrow for history navigation
      if (e.altKey) {
        if (e.key === "ArrowLeft") {
          e.preventDefault();
          goBack(paneId);
        } else if (e.key === "ArrowRight") {
          e.preventDefault();
          goForward(paneId);
        }
      }
    },
    [
      paneId,
      goUp,
      refresh,
      goBack,
      goForward,
      listing,
      setSelection,
      handleCopy,
      handleCut,
      handlePaste,
      handleRename,
      handleDelete,
      handleShowProperties,
      handleNewFolder,
      getSelectedEntries,
      handleDoubleClick,
    ]
  );

  const entries = listing?.entries ?? [];

  return (
    <>
      <section
        ref={setDropRef}
        className={clsx(
          "flex flex-1 flex-col overflow-hidden",
          isActive ? "ring-1 ring-primary/50" : "",
          showDropHighlight && "drop-zone-active"
        )}
        onClick={() => setActivePane(paneId)}
        onKeyDown={handleKeyDown}
        onContextMenu={(e) => handleContextMenu(e)}
        aria-label={`${paneId} file pane`}
      >
        <PaneHeader
          paneId={paneId}
          path={path}
          canGoBack={historyBack.length > 0}
          canGoForward={historyForward.length > 0}
          searchQuery={searchQuery}
          onBack={() => goBack(paneId)}
          onForward={() => goForward(paneId)}
          onUp={() => goUp(paneId)}
          onRefresh={() => refresh(paneId)}
          onNavigate={(newPath) => navigateTo(paneId, newPath)}
          onSearchChange={handleSearchChange}
          onNewFolder={handleNewFolder}
          onNewFile={handleCreateFile}
        />

        <VirtualizedFileList
          entries={entries}
          selectedIndices={selectedIndices}
          cursorIndex={cursorIndex}
          sort={sort}
          paneId={paneId}
          isLoading={isLoading}
          error={error}
          onSelect={handleSelect}
          onDoubleClick={handleDoubleClick}
          onSortChange={handleSortChange}
          onRetry={() => refresh(paneId)}
          onContextMenu={(e, entry) => handleContextMenu(e, entry)}
          onGoUp={() => goUp(paneId)}
          onRefresh={() => refresh(paneId)}
        />
      </section>

      {/* Properties panel modal */}
      {propertiesEntries.length > 0 && (
        <PropertiesPanel
          entries={propertiesEntries}
          onClose={() => setPropertiesEntries([])}
          asModal
        />
      )}
    </>
  );
}
