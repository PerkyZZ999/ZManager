/**
 * DndProvider component
 *
 * Provides drag-and-drop functionality using @dnd-kit/core for internal DnD
 * and Tauri APIs for external file drops from/to the OS.
 *
 * Features:
 * - Internal drag between file panes
 * - Drop zone highlighting
 * - External file drop from Explorer (onDragDropEvent)
 * - External drag-out to Explorer (startDrag)
 */

import {
  DndContext,
  type DragEndEvent,
  type DragOverEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  pointerWithin,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from "react";
import type { PaneId } from "../stores";
import type { EntryMeta } from "../types";
import { getIconForEntry } from "../utils/iconMappings";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

export interface DragData {
  type: "file-entries";
  entries: EntryMeta[];
  sourcePaneId: PaneId;
}

export interface DropZoneData {
  type: "pane";
  paneId: PaneId;
  path: string;
}

interface DndContextValue {
  /** Currently dragged entries */
  draggedEntries: EntryMeta[] | null;
  /** Source pane of the current drag */
  sourcePaneId: PaneId | null;
  /** Pane ID currently being hovered over */
  overPaneId: PaneId | null;
  /** Whether an external drop is happening */
  isExternalDrop: boolean;
  /** External files being dropped */
  externalFiles: string[];
  /** Start dragging files out to the OS */
  startExternalDrag: (entries: EntryMeta[]) => Promise<void>;
}

const DndContextInstance = createContext<DndContextValue | null>(null);

// ============================================================================
// Drag Overlay Component
// ============================================================================

function DragPreview({ entries }: { entries: EntryMeta[] }) {
  if (entries.length === 0) return null;

  const firstEntry = entries[0];
  const iconInfo = getIconForEntry(firstEntry);

  return (
    <div className="flex items-center gap-2 rounded bg-zinc-800/95 px-3 py-2 shadow-lg ring-1 ring-white/10">
      <SvgIcon src={iconInfo.path} size={20} alt={firstEntry.kind} />
      <span className="max-w-48 truncate font-medium text-sm">
        {entries.length === 1 ? firstEntry.name : `${entries.length} items`}
      </span>
    </div>
  );
}

// ============================================================================
// Provider Component
// ============================================================================

interface DndProviderProps {
  children: ReactNode;
  onFileDrop?: (files: string[], targetPane: PaneId, targetPath: string) => void;
  onInternalMove?: (
    entries: EntryMeta[],
    sourcePaneId: PaneId,
    targetPaneId: PaneId,
    targetPath: string
  ) => void;
}

export function DndProvider({
  children,
  onFileDrop: _onFileDrop,
  onInternalMove,
}: DndProviderProps) {
  // Drag state
  const [draggedEntries, setDraggedEntries] = useState<EntryMeta[] | null>(null);
  const [sourcePaneId, setSourcePaneId] = useState<PaneId | null>(null);
  const [overPaneId, setOverPaneId] = useState<PaneId | null>(null);

  // External drop state
  const [isExternalDrop, setIsExternalDrop] = useState(false);
  const [externalFiles, setExternalFiles] = useState<string[]>([]);

  // Configure pointer sensor with activation constraint
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // Minimum drag distance before activation
      },
    })
  );

  // Handle drag start
  const handleDragStart = useCallback((event: DragStartEvent) => {
    const data = event.active.data.current as DragData | undefined;
    if (data?.type === "file-entries") {
      setDraggedEntries(data.entries);
      setSourcePaneId(data.sourcePaneId);
    }
  }, []);

  // Handle drag over
  const handleDragOver = useCallback((event: DragOverEvent) => {
    const overData = event.over?.data.current as DropZoneData | undefined;
    if (overData?.type === "pane") {
      setOverPaneId(overData.paneId);
    } else {
      setOverPaneId(null);
    }
  }, []);

  // Handle drag end
  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active: _active, over } = event;

      if (over && draggedEntries && sourcePaneId) {
        const overData = over.data.current as DropZoneData | undefined;
        if (overData?.type === "pane" && overData.paneId !== sourcePaneId) {
          // Internal move/copy between panes
          onInternalMove?.(draggedEntries, sourcePaneId, overData.paneId, overData.path);
        }
      }

      // Reset state
      setDraggedEntries(null);
      setSourcePaneId(null);
      setOverPaneId(null);
    },
    [draggedEntries, sourcePaneId, onInternalMove]
  );

  // Handle drag cancel
  const handleDragCancel = useCallback(() => {
    setDraggedEntries(null);
    setSourcePaneId(null);
    setOverPaneId(null);
  }, []);

  // Start external drag (to Explorer)
  const startExternalDrag = useCallback(async (entries: EntryMeta[]) => {
    try {
      const { startDrag } = await import("@crabnebula/tauri-plugin-drag");
      const paths = entries.map((e) => e.path);
      // Use first file's icon as preview (or a generic icon)
      const iconPath = entries[0]?.path ?? "";

      await startDrag(
        {
          item: paths,
          icon: iconPath,
        },
        (_result) => {
          // Drag completed - result is "Dropped" or "Cancelled"
        }
      );
    } catch (_err) {
      // External drag failed - silently ignore
    }
  }, []);

  // Listen for external file drops from OS
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      try {
        const webview = getCurrentWebview();
        unlisten = await webview.onDragDropEvent((event) => {
          // In Tauri v2, event is Event<DragDropEvent> where DragDropEvent is the payload
          const dragEvent = event.payload;
          switch (dragEvent.type) {
            case "enter":
            case "over":
              setIsExternalDrop(true);
              break;
            case "drop":
              setIsExternalDrop(false);
              if (dragEvent.paths && dragEvent.paths.length > 0) {
                setExternalFiles(dragEvent.paths);
                // The actual drop handling is done by the drop zone components
                // They will call onFileDrop with the target pane info
              }
              break;
            case "leave":
              setIsExternalDrop(false);
              setExternalFiles([]);
              break;
          }
        });
      } catch (_err) {
        // Failed to setup drag-drop listener - silently ignore
      }
    };

    setupListener();

    return () => {
      unlisten?.();
    };
  }, []);

  // Context value
  const contextValue: DndContextValue = {
    draggedEntries,
    sourcePaneId,
    overPaneId,
    isExternalDrop,
    externalFiles,
    startExternalDrag,
  };

  return (
    <DndContextInstance.Provider value={contextValue}>
      <DndContext
        sensors={sensors}
        collisionDetection={pointerWithin}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
        onDragCancel={handleDragCancel}
      >
        {children}
        <DragOverlay dropAnimation={null}>
          {draggedEntries && <DragPreview entries={draggedEntries} />}
        </DragOverlay>
      </DndContext>
    </DndContextInstance.Provider>
  );
}

// ============================================================================
// Hook
// ============================================================================

export function useDnd() {
  const context = useContext(DndContextInstance);
  if (!context) {
    throw new Error("useDnd must be used within a DndProvider");
  }
  return context;
}
