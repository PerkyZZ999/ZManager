/**
 * Sidebar component
 *
 * Displays:
 * - Quick Access / Favorites section (drag-to-reorder, right-click remove)
 * - Drives section
 */

import {
  closestCenter,
  DndContext,
  type DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  type FavoriteDto,
  type SidebarSection,
  useFavoritesStore,
  useFileSystemStore,
  useUIStore,
} from "../stores";
import type { DriveInfo } from "../types";
import { getDriveIconName, getUiIconName } from "../utils/iconMappings";
import { SvgIcon } from "./SvgIcon";

/** Format bytes to human-readable string */
function formatBytes(bytes: number | null): string {
  if (bytes === null) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let unitIndex = 0;
  let value = bytes;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}

/** Collapsible section header */
function SectionHeader({
  title,
  section,
  isExpanded,
  onToggle,
}: {
  title: string;
  section: SidebarSection;
  isExpanded: boolean;
  onToggle: (section: SidebarSection) => void;
}) {
  return (
    <button
      type="button"
      onClick={() => onToggle(section)}
      className="flex w-full items-center gap-2 px-3 py-2 text-left font-semibold text-xs text-zinc-400 uppercase tracking-wider hover:text-zinc-200"
    >
      <span className={`transition-transform ${isExpanded ? "rotate-90" : ""}`}>
        <SvgIcon name={getUiIconName("chevron_right")} size={12} />
      </span>
      {title}
    </button>
  );
}

/** Drive item component */
function DriveItem({ drive, onClick }: { drive: DriveInfo; onClick: (path: string) => void }) {
  const usagePercent =
    drive.total_bytes && drive.free_bytes
      ? ((drive.total_bytes - drive.free_bytes) / drive.total_bytes) * 100
      : null;

  const driveIconName = getDriveIconName(drive.drive_type);

  return (
    <button
      type="button"
      onClick={() => onClick(drive.path)}
      className="flex w-full flex-col gap-1 rounded px-3 py-2 text-left hover:bg-white/5"
    >
      <div className="flex items-center gap-2">
        <SvgIcon name={driveIconName} size={16} />
        <span className="flex-1 truncate text-sm">
          {drive.label || "Local Disk"} ({drive.path.replace("\\", "")})
        </span>
      </div>
      {usagePercent !== null && (
        <div className="ml-6 flex items-center gap-2">
          <div className="h-1 flex-1 overflow-hidden rounded-full bg-zinc-700">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${Math.min(usagePercent, 100)}%` }}
            />
          </div>
          <span className="text-xs text-zinc-500">{formatBytes(drive.free_bytes)} free</span>
        </div>
      )}
    </button>
  );
}

/** Context menu component for favorites */
function FavoriteContextMenu({
  x,
  y,
  onRemove,
  onClose,
}: {
  x: number;
  y: number;
  onRemove: () => void;
  onClose: () => void;
}) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose]);

  return (
    <div
      ref={menuRef}
      className="fixed z-50 min-w-32 rounded border border-zinc-600 bg-zinc-800 py-1 shadow-lg"
      style={{ left: x, top: y }}
    >
      <button
        type="button"
        onClick={() => {
          onRemove();
          onClose();
        }}
        className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-red-400 text-sm hover:bg-white/5"
      >
        <SvgIcon name={getUiIconName("close")} size={14} />
        Remove from Quick Access
      </button>
    </div>
  );
}

/** Sortable favorite item component */
function SortableFavoriteItem({
  favorite,
  onClick,
  onContextMenu,
}: {
  favorite: FavoriteDto;
  onClick: (path: string) => void;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: favorite.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  // Map icon name to symbol name, with fallback to folder
  const iconName = favorite.icon ? getUiIconName(favorite.icon) : getUiIconName("folder");

  return (
    <button
      ref={setNodeRef}
      type="button"
      style={style}
      {...attributes}
      {...listeners}
      onClick={() => onClick(favorite.path)}
      onContextMenu={(e) => onContextMenu(e, favorite.id)}
      className={`flex w-full items-center gap-2 rounded px-3 py-1.5 text-left text-sm hover:bg-white/5 ${
        !favorite.is_valid ? "text-zinc-500 line-through" : ""
      }`}
    >
      <SvgIcon name={iconName} size={16} />
      <span className="truncate">{favorite.name}</span>
    </button>
  );
}

/** Default favorites for new installations */
const DEFAULT_FAVORITES: Array<{ name: string; pathSuffix: string; icon: string }> = [
  { name: "Home", pathSuffix: "", icon: "home" },
  { name: "Desktop", pathSuffix: "\\Desktop", icon: "desktop" },
  { name: "Downloads", pathSuffix: "\\Downloads", icon: "arrow_download" },
  { name: "Documents", pathSuffix: "\\Documents", icon: "document" },
];

export function Sidebar() {
  const { sidebarVisible, expandedSections, toggleSection } = useUIStore();
  const { drives, drivesLoading, loadDrives, navigateTo, activePane } = useFileSystemStore();
  const {
    favorites,
    isLoading: favoritesLoading,
    loadFavorites,
    removeFavorite,
    reorderFavorites,
    addFavorite,
  } = useFavoritesStore();

  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; id: string } | null>(null);

  // DnD sensors for sortable favorites
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  // Load data on mount
  useEffect(() => {
    loadDrives();
    loadFavorites();
  }, [loadDrives, loadFavorites]);

  // Add default favorites if none exist (first run)
  // Use ref to prevent double execution in React Strict Mode
  const defaultsAddedRef = useRef(false);
  useEffect(() => {
    // Only add defaults when not loading and no favorites exist yet
    if (favoritesLoading || favorites.length > 0 || defaultsAddedRef.current) {
      return;
    }
    defaultsAddedRef.current = true;

    // Get user profile path from environment
    const userProfile =
      import.meta.env.VITE_USERPROFILE || `C:\\Users\\${import.meta.env.VITE_USERNAME || "Public"}`;

    // Add defaults sequentially
    const addDefaults = async () => {
      for (const def of DEFAULT_FAVORITES) {
        const path = def.pathSuffix ? `${userProfile}${def.pathSuffix}` : userProfile;
        await addFavorite(def.name, path, def.icon);
      }
    };
    addDefaults();
  }, [favoritesLoading, favorites.length, addFavorite]);

  const handleNavigate = useCallback(
    (path: string) => {
      navigateTo(activePane, path);
    },
    [navigateTo, activePane]
  );

  const handleContextMenu = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, id });
  }, []);

  const handleRemoveFavorite = useCallback(() => {
    if (contextMenu) {
      removeFavorite(contextMenu.id);
    }
  }, [contextMenu, removeFavorite]);

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      if (over && active.id !== over.id) {
        const oldIndex = favorites.findIndex((f) => f.id === active.id);
        const newIndex = favorites.findIndex((f) => f.id === over.id);

        // Create new order
        const newFavorites = [...favorites];
        const [moved] = newFavorites.splice(oldIndex, 1);
        newFavorites.splice(newIndex, 0, moved);

        // Submit reorder
        reorderFavorites(newFavorites.map((f) => f.id));
      }
    },
    [favorites, reorderFavorites]
  );

  if (!sidebarVisible) return null;

  return (
    <aside className="flex w-56 flex-col border-zinc-700 border-r bg-zinc-800">
      {/* Quick Access / Favorites Section */}
      <div>
        <SectionHeader
          title="Quick Access"
          section="favorites"
          isExpanded={expandedSections.has("favorites")}
          onToggle={toggleSection}
        />
        {expandedSections.has("favorites") && (
          <div className="pb-2">
            {favoritesLoading ? (
              <div className="px-3 py-2 text-sm text-zinc-500">Loading...</div>
            ) : favorites.length === 0 ? (
              <div className="px-3 py-2 text-sm text-zinc-500">No favorites yet</div>
            ) : (
              <DndContext
                sensors={sensors}
                collisionDetection={closestCenter}
                modifiers={[restrictToVerticalAxis]}
                onDragEnd={handleDragEnd}
              >
                <SortableContext
                  items={favorites.map((f) => f.id)}
                  strategy={verticalListSortingStrategy}
                >
                  {favorites.map((fav) => (
                    <SortableFavoriteItem
                      key={fav.id}
                      favorite={fav}
                      onClick={handleNavigate}
                      onContextMenu={handleContextMenu}
                    />
                  ))}
                </SortableContext>
              </DndContext>
            )}
          </div>
        )}
      </div>

      {/* Drives Section */}
      <div>
        <SectionHeader
          title="Drives"
          section="drives"
          isExpanded={expandedSections.has("drives")}
          onToggle={toggleSection}
        />
        {expandedSections.has("drives") && (
          <div className="pb-2">
            {drivesLoading ? (
              <div className="px-3 py-2 text-sm text-zinc-500">Loading...</div>
            ) : drives.length === 0 ? (
              <div className="px-3 py-2 text-sm text-zinc-500">No drives found</div>
            ) : (
              drives.map((drive) => (
                <DriveItem key={drive.path} drive={drive} onClick={handleNavigate} />
              ))
            )}
          </div>
        )}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Footer with version */}
      <div className="border-zinc-700 border-t px-3 py-2 text-xs text-zinc-500">
        ZManager v0.1.0
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <FavoriteContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          onRemove={handleRemoveFavorite}
          onClose={() => setContextMenu(null)}
        />
      )}
    </aside>
  );
}
