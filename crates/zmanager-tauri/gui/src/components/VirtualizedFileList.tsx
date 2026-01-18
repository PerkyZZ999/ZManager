/**
 * VirtualizedFileList component
 *
 * Renders a virtualized file list using @tanstack/react-virtual.
 * Designed to handle 50k+ entries smoothly.
 */

import { useDraggable } from "@dnd-kit/core";
import { useVirtualizer } from "@tanstack/react-virtual";
import clsx from "clsx";
import { useCallback, useMemo, useRef } from "react";
import type { PaneId } from "../stores";
import type { EntryMeta, SortField, SortSpec } from "../types";
import { getIconForEntry } from "../utils/iconMappings";
import type { DragData } from "./DndProvider";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Constants
// ============================================================================

const ROW_HEIGHT = 28; // pixels per row
const OVERSCAN = 5; // extra rows to render above/below viewport

// ============================================================================
// Utility Functions
// ============================================================================

/** Format bytes to human-readable string */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let unitIndex = 0;
  let value = bytes;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return unitIndex === 0
    ? `${Math.round(value)} ${units[unitIndex]}`
    : `${value.toFixed(1)} ${units[unitIndex]}`;
}

/** Format date to locale string */
function formatDate(dateStr: string | null): string {
  if (!dateStr) return "—";
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return "—";
  }
}

// ============================================================================
// Column Header Component
// ============================================================================

interface ColumnHeaderProps {
  sort: SortSpec;
  onSortChange: (field: SortField) => void;
}

function ColumnHeader({ sort, onSortChange }: ColumnHeaderProps) {
  const renderSortIndicator = (field: SortField) => {
    if (sort.field !== field) return null;
    return (
      <SvgIcon
        name={sort.order === "ascending" ? "ic_arrow_sort_up" : "ic_arrow_sort_down"}
        size={12}
        className="ml-1"
      />
    );
  };

  const headerButton = (field: SortField, label: string, className: string) => (
    <button
      type="button"
      onClick={() => onSortChange(field)}
      className={clsx(
        className,
        "flex items-center transition-colors hover:text-white",
        sort.field === field && "text-primary"
      )}
    >
      {label}
      {renderSortIndicator(field)}
    </button>
  );

  return (
    <div className="flex items-center gap-2 border-zinc-700 border-b bg-zinc-800/50 px-3 py-1.5 font-semibold text-xs text-zinc-400 uppercase tracking-wider">
      <span className="w-5" aria-hidden="true" />
      {headerButton("name", "Name", "flex-1")}
      {headerButton("size", "Size", "w-20 justify-end")}
      {headerButton("modified", "Modified", "w-36 justify-end")}
    </div>
  );
}

// ============================================================================
// File Row Component
// ============================================================================

interface FileRowProps {
  entry: EntryMeta;
  index: number;
  style: React.CSSProperties;
  isSelected: boolean;
  isCursor: boolean;
  paneId: PaneId;
  selectedEntries: EntryMeta[];
  onSelect: (index: number, entry: EntryMeta, event: React.MouseEvent) => void;
  onDoubleClick: (entry: EntryMeta) => void;
  onContextMenu?: (event: React.MouseEvent, entry: EntryMeta) => void;
}

function FileRow({
  entry,
  index,
  style,
  isSelected,
  isCursor,
  paneId,
  selectedEntries,
  onSelect,
  onDoubleClick,
  onContextMenu,
}: FileRowProps) {
  const iconInfo = getIconForEntry(entry);

  // Set up draggable - drag the selected entries when this row is dragged
  const dragData: DragData = {
    type: "file-entries",
    entries: isSelected ? selectedEntries : [entry],
    sourcePaneId: paneId,
  };

  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `file-${entry.path}`,
    data: dragData,
  });

  return (
    <li
      ref={setNodeRef}
      style={style}
      onClick={(e) => onSelect(index, entry, e)}
      onDoubleClick={() => onDoubleClick(entry)}
      onContextMenu={(e) => onContextMenu?.(e, entry)}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          onDoubleClick(entry);
        }
      }}
      className={clsx(
        "absolute right-0 left-0 flex cursor-pointer items-center gap-2 px-3 text-sm",
        "border-l-2",
        isSelected ? "border-primary bg-primary/20" : "border-transparent",
        isCursor && !isSelected && "bg-white/5",
        "hover:bg-white/10",
        entry.attributes.hidden && "opacity-60",
        isDragging && "opacity-50"
      )}
      data-index={index}
      {...attributes}
      {...listeners}
      tabIndex={0}
    >
      <span className="flex w-5 shrink-0 items-center justify-center">
        <SvgIcon name={iconInfo.symbolName} size={16} />
      </span>
      <span className="min-w-0 flex-1 truncate">{entry.name}</span>
      <span className="w-20 shrink-0 text-right text-zinc-400">
        {entry.kind === "directory" ? "—" : formatBytes(entry.size)}
      </span>
      <span className="w-36 shrink-0 text-right text-zinc-500">{formatDate(entry.modified)}</span>
    </li>
  );
}

// ============================================================================
// Empty State Component
// ============================================================================

function EmptyState() {
  return (
    <div className="flex h-full flex-col items-center justify-center py-16 text-zinc-500">
      <SvgIcon name="folder-base-open" size={64} className="mb-4 opacity-30" />
      <p className="font-medium text-lg">This folder is empty</p>
      <p className="mt-1 text-sm">Drop files here or create new ones</p>
    </div>
  );
}

// ============================================================================
// Loading State Component
// ============================================================================

function LoadingState() {
  return (
    <div className="flex items-center justify-center py-16 text-zinc-400">
      <div className="flex items-center gap-3">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <span>Loading...</span>
      </div>
    </div>
  );
}

// ============================================================================
// Error State Component
// ============================================================================

interface ErrorStateProps {
  error: string;
  onRetry: () => void;
}

function ErrorState({ error, onRetry }: ErrorStateProps) {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-16">
      <div className="flex items-center gap-2 text-red-400">
        <SvgIcon name="ic_warning" size={24} />
        <span>{error}</span>
      </div>
      <button
        type="button"
        onClick={onRetry}
        className="rounded bg-zinc-700 px-4 py-2 font-medium text-sm transition-colors hover:bg-zinc-600"
      >
        Retry
      </button>
    </div>
  );
}

// ============================================================================
// Main VirtualizedFileList Component
// ============================================================================

export interface VirtualizedFileListProps {
  entries: EntryMeta[];
  selectedIndices: Set<number>;
  cursorIndex: number;
  sort: SortSpec;
  /** Pane identifier for drag & drop operations */
  paneId: PaneId;
  isLoading?: boolean;
  error?: string | null;
  onSelect: (index: number, entry: EntryMeta, event: React.MouseEvent) => void;
  onDoubleClick: (entry: EntryMeta) => void;
  onSortChange: (field: SortField) => void;
  onRetry?: () => void;
  onContextMenu?: (event: React.MouseEvent, entry: EntryMeta) => void;
  /** Called when Backspace is pressed to navigate to parent */
  onGoUp?: () => void;
  /** Called when F5 is pressed to refresh */
  onRefresh?: () => void;
}

export function VirtualizedFileList({
  entries,
  selectedIndices,
  cursorIndex,
  sort,
  paneId,
  isLoading = false,
  error = null,
  onSelect,
  onDoubleClick,
  onSortChange,
  onRetry,
  onContextMenu,
  onGoUp,
  onRefresh,
}: VirtualizedFileListProps) {
  const parentRef = useRef<HTMLDivElement>(null);

  // Pre-compute selected entries for drag data
  const selectedEntries = useMemo(
    () => entries.filter((_, idx) => selectedIndices.has(idx)),
    [entries, selectedIndices]
  );

  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: OVERSCAN,
  });

  const virtualItems = virtualizer.getVirtualItems();

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Handle navigation keys that don't require entries
      switch (e.key) {
        case "Backspace":
          e.preventDefault();
          onGoUp?.();
          return;
        case "F5":
          e.preventDefault();
          onRefresh?.();
          return;
      }

      if (entries.length === 0) return;

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          if (cursorIndex < entries.length - 1) {
            const newIndex = cursorIndex + 1;
            onSelect(newIndex, entries[newIndex], e as unknown as React.MouseEvent);
            virtualizer.scrollToIndex(newIndex, { align: "auto" });
          }
          break;
        case "ArrowUp":
          e.preventDefault();
          if (cursorIndex > 0) {
            const newIndex = cursorIndex - 1;
            onSelect(newIndex, entries[newIndex], e as unknown as React.MouseEvent);
            virtualizer.scrollToIndex(newIndex, { align: "auto" });
          }
          break;
        case "Home":
          e.preventDefault();
          onSelect(0, entries[0], e as unknown as React.MouseEvent);
          virtualizer.scrollToIndex(0, { align: "start" });
          break;
        case "End": {
          e.preventDefault();
          const lastIndex = entries.length - 1;
          onSelect(lastIndex, entries[lastIndex], e as unknown as React.MouseEvent);
          virtualizer.scrollToIndex(lastIndex, { align: "end" });
          break;
        }
        case "PageDown":
          e.preventDefault();
          {
            const visibleCount = Math.floor((parentRef.current?.clientHeight ?? 400) / ROW_HEIGHT);
            const newIndex = Math.min(cursorIndex + visibleCount, entries.length - 1);
            onSelect(newIndex, entries[newIndex], e as unknown as React.MouseEvent);
            virtualizer.scrollToIndex(newIndex, { align: "auto" });
          }
          break;
        case "PageUp":
          e.preventDefault();
          {
            const visibleCount = Math.floor((parentRef.current?.clientHeight ?? 400) / ROW_HEIGHT);
            const newIndex = Math.max(cursorIndex - visibleCount, 0);
            onSelect(newIndex, entries[newIndex], e as unknown as React.MouseEvent);
            virtualizer.scrollToIndex(newIndex, { align: "auto" });
          }
          break;
        case "Enter":
          e.preventDefault();
          if (cursorIndex >= 0 && cursorIndex < entries.length) {
            onDoubleClick(entries[cursorIndex]);
          }
          break;
      }
    },
    [entries, cursorIndex, onSelect, onDoubleClick, virtualizer, onGoUp, onRefresh]
  );

  // Loading state
  if (isLoading) {
    return (
      <div className="flex flex-1 flex-col overflow-hidden">
        <ColumnHeader sort={sort} onSortChange={onSortChange} />
        <LoadingState />
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex flex-1 flex-col overflow-hidden">
        <ColumnHeader sort={sort} onSortChange={onSortChange} />
        <ErrorState error={error} onRetry={onRetry ?? (() => {})} />
      </div>
    );
  }

  // Empty state
  if (entries.length === 0) {
    return (
      <div className="flex flex-1 flex-col overflow-hidden">
        <ColumnHeader sort={sort} onSortChange={onSortChange} />
        <EmptyState />
      </div>
    );
  }

  // Normal virtualized list
  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <ColumnHeader sort={sort} onSortChange={onSortChange} />
      <section
        ref={parentRef}
        className="flex-1 overflow-auto bg-zinc-850 focus:outline-none"
        tabIndex={0}
        onKeyDown={handleKeyDown}
        aria-label="File list"
      >
        <ul
          className="relative list-none"
          style={{ height: virtualizer.getTotalSize() }}
          aria-label="File entries"
        >
          {virtualItems.map((virtualItem) => {
            const entry = entries[virtualItem.index];
            return (
              <FileRow
                key={entry.path}
                entry={entry}
                index={virtualItem.index}
                style={{
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                }}
                isSelected={selectedIndices.has(virtualItem.index)}
                isCursor={cursorIndex === virtualItem.index}
                paneId={paneId}
                selectedEntries={selectedEntries}
                onSelect={onSelect}
                onDoubleClick={onDoubleClick}
                onContextMenu={onContextMenu}
              />
            );
          })}
        </ul>
      </section>
    </div>
  );
}
