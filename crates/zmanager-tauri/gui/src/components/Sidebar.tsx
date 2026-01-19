/**
 * Sidebar component
 *
 * Displays:
 * - Quick Access / Favorites section (drag-to-reorder, right-click remove)
 * - Drives section
 * - Footer with Settings, View, About buttons
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
  type ViewMode,
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

// ============================================================================
// View Dropdown Component (opens upward)
// ============================================================================

interface ViewDropdownProps {
  isOpen: boolean;
  onClose: () => void;
  viewMode: ViewMode;
  onModeChange: (mode: ViewMode) => void;
}

function ViewDropdown({ isOpen, onClose, viewMode, onModeChange }: ViewDropdownProps) {
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div
      ref={dropdownRef}
      className="absolute bottom-full left-0 mb-1 min-w-28 rounded border border-zinc-600 bg-zinc-800 py-1 shadow-lg"
    >
      <button
        type="button"
        onClick={() => {
          onModeChange("list");
          onClose();
        }}
        className={`flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm hover:bg-white/5 ${
          viewMode === "list" ? "text-primary" : ""
        }`}
      >
        <SvgIcon name={getUiIconName("list")} size={14} />
        List
        {viewMode === "list" && (
          <SvgIcon name={getUiIconName("checkmark")} size={12} className="ml-auto" />
        )}
      </button>
      <button
        type="button"
        onClick={() => {
          onModeChange("grid");
          onClose();
        }}
        className={`flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm hover:bg-white/5 ${
          viewMode === "grid" ? "text-primary" : ""
        }`}
      >
        <SvgIcon name={getUiIconName("grid")} size={14} />
        Grid
        {viewMode === "grid" && (
          <SvgIcon name={getUiIconName("checkmark")} size={12} className="ml-auto" />
        )}
      </button>
    </div>
  );
}

// ============================================================================
// Settings Dialog Component
// ============================================================================

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-200 flex items-center justify-center bg-black/60">
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        className="w-125 overflow-hidden rounded-lg border border-zinc-700 bg-zinc-800 shadow-2xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-zinc-700 border-b px-4 py-3">
          <h2 className="font-semibold text-lg">Settings</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 transition-colors hover:bg-white/10"
            aria-label="Close dialog"
          >
            <SvgIcon name={getUiIconName("close")} size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="px-4 py-6">
          <div className="space-y-6">
            {/* General Section */}
            <div>
              <h3 className="mb-3 font-medium text-sm text-zinc-300">General</h3>
              <div className="space-y-3">
                <label className="flex items-center justify-between">
                  <span className="text-sm text-zinc-400">Show hidden files</span>
                  <input type="checkbox" className="h-4 w-4 rounded accent-primary" />
                </label>
                <label className="flex items-center justify-between">
                  <span className="text-sm text-zinc-400">Confirm before delete</span>
                  <input
                    type="checkbox"
                    defaultChecked
                    className="h-4 w-4 rounded accent-primary"
                  />
                </label>
              </div>
            </div>

            {/* Appearance Section */}
            <div>
              <h3 className="mb-3 font-medium text-sm text-zinc-300">Appearance</h3>
              <div className="space-y-3">
                <label className="flex items-center justify-between">
                  <span className="text-sm text-zinc-400">Theme</span>
                  <select className="rounded bg-zinc-700 px-2 py-1 text-sm outline-none">
                    <option value="dark">Dark</option>
                    <option value="light" disabled>
                      Light (Coming Soon)
                    </option>
                  </select>
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 border-zinc-700 border-t px-4 py-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded bg-zinc-700 px-4 py-2 font-medium text-sm transition-colors hover:bg-zinc-600"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// About Dialog Component
// ============================================================================

interface AboutDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

function AboutDialog({ isOpen, onClose }: AboutDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-200 flex items-center justify-center bg-black/60">
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        className="w-80 overflow-hidden rounded-lg border border-zinc-700 bg-zinc-800 shadow-2xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-zinc-700 border-b px-4 py-3">
          <h2 className="font-semibold text-lg">About</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 transition-colors hover:bg-white/10"
            aria-label="Close dialog"
          >
            <SvgIcon name={getUiIconName("close")} size={16} />
          </button>
        </div>

        {/* Content */}
        <div className="flex flex-col items-center px-4 py-6 text-center">
          <SvgIcon name={getUiIconName("folder")} size={48} className="mb-4 text-primary" />
          <h3 className="font-bold text-xl">ZManager</h3>
          <p className="mt-1 text-sm text-zinc-400">Version 0.1.0</p>
          <p className="mt-4 text-sm text-zinc-500">A modern dual-pane file manager for Windows.</p>
          <p className="mt-2 text-xs text-zinc-600">Built with Rust, Tauri & React</p>
        </div>

        {/* Footer */}
        <div className="flex justify-center border-zinc-700 border-t px-4 py-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded bg-primary px-4 py-2 font-medium text-sm text-zinc-900 transition-colors hover:bg-primary/80"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Sidebar Footer Component
// ============================================================================

interface SidebarFooterProps {
  onSettingsClick: () => void;
  onAboutClick: () => void;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
}

function SidebarFooter({
  onSettingsClick,
  onAboutClick,
  viewMode,
  onViewModeChange,
}: SidebarFooterProps) {
  const [viewDropdownOpen, setViewDropdownOpen] = useState(false);

  return (
    <div className="relative flex items-center justify-center gap-1 border-zinc-700 border-t px-2 py-2">
      {/* Settings Button - icon only with tooltip */}
      <button
        type="button"
        onClick={onSettingsClick}
        className="flex items-center justify-center rounded p-2 text-zinc-400 transition-colors hover:bg-white/5 hover:text-zinc-200"
        title="Settings"
        aria-label="Settings"
      >
        <SvgIcon name={getUiIconName("settings")} size={18} />
      </button>

      {/* View Button with Dropdown - icon only with tooltip */}
      <div className="relative">
        <button
          type="button"
          onClick={() => setViewDropdownOpen(!viewDropdownOpen)}
          className="flex items-center justify-center rounded p-2 text-zinc-400 transition-colors hover:bg-white/5 hover:text-zinc-200"
          title="View"
          aria-label="View"
        >
          <SvgIcon
            name={viewMode === "list" ? getUiIconName("list") : getUiIconName("grid")}
            size={18}
          />
        </button>
        <ViewDropdown
          isOpen={viewDropdownOpen}
          onClose={() => setViewDropdownOpen(false)}
          viewMode={viewMode}
          onModeChange={onViewModeChange}
        />
      </div>

      {/* About Button - icon only with tooltip */}
      <button
        type="button"
        onClick={onAboutClick}
        className="flex items-center justify-center rounded p-2 text-zinc-400 transition-colors hover:bg-white/5 hover:text-zinc-200"
        title="About"
        aria-label="About"
      >
        <SvgIcon name={getUiIconName("info")} size={18} />
      </button>
    </div>
  );
}

/** Default favorites are now created in the backend when config is first created */

export function Sidebar() {
  const { sidebarVisible, expandedSections, toggleSection, viewMode, setViewMode } = useUIStore();
  const { drives, drivesLoading, loadDrives, navigateTo, activePane } = useFileSystemStore();
  const {
    favorites,
    isLoading: favoritesLoading,
    loadFavorites,
    removeFavorite,
    reorderFavorites,
  } = useFavoritesStore();

  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; id: string } | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);

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

      {/* Footer with action buttons */}
      <SidebarFooter
        onSettingsClick={() => setSettingsOpen(true)}
        onAboutClick={() => setAboutOpen(true)}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
      />

      {/* Context Menu */}
      {contextMenu && (
        <FavoriteContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          onRemove={handleRemoveFavorite}
          onClose={() => setContextMenu(null)}
        />
      )}

      {/* Dialogs */}
      <SettingsDialog isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <AboutDialog isOpen={aboutOpen} onClose={() => setAboutOpen(false)} />
    </aside>
  );
}
