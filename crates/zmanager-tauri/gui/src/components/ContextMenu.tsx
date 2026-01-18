/**
 * ContextMenu component
 *
 * A floating context menu that appears on right-click.
 * Supports nested menus, icons, keyboard shortcuts, and separators.
 */

import clsx from "clsx";
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

export interface MenuItemDef {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  onClick?: () => void;
  submenu?: MenuItemDef[];
}

export interface MenuSeparator {
  separator: true;
}

export type MenuEntry = MenuItemDef | MenuSeparator;

interface ContextMenuState {
  x: number;
  y: number;
  items: MenuEntry[];
}

interface ContextMenuContextValue {
  show: (x: number, y: number, items: MenuEntry[]) => void;
  hide: () => void;
}

// ============================================================================
// Context
// ============================================================================

const ContextMenuContext = createContext<ContextMenuContextValue | null>(null);

export function useContextMenu() {
  const ctx = useContext(ContextMenuContext);
  if (!ctx) {
    throw new Error("useContextMenu must be used within ContextMenuProvider");
  }
  return ctx;
}

// ============================================================================
// Menu Item Component
// ============================================================================

function MenuItem({ item, onClose }: { item: MenuItemDef; onClose: () => void }) {
  const handleClick = useCallback(() => {
    if (item.disabled) return;
    item.onClick?.();
    onClose();
  }, [item, onClose]);

  return (
    <button
      type="button"
      onClick={handleClick}
      disabled={item.disabled}
      className={clsx(
        "flex w-full items-center gap-3 px-3 py-1.5 text-left text-sm",
        "transition-colors",
        item.disabled
          ? "cursor-not-allowed text-zinc-500"
          : item.danger
            ? "text-red-400 hover:bg-red-500/20"
            : "text-zinc-200 hover:bg-white/10"
      )}
    >
      {/* Icon */}
      <span className="flex w-4 items-center justify-center">
        {item.icon && <SvgIcon name={item.icon} size={16} />}
      </span>

      {/* Label */}
      <span className="flex-1">{item.label}</span>

      {/* Shortcut */}
      {item.shortcut && <span className="text-xs text-zinc-500">{item.shortcut}</span>}
    </button>
  );
}

// ============================================================================
// Menu Component
// ============================================================================

function Menu({
  items,
  onClose,
  style,
}: {
  items: MenuEntry[];
  onClose: () => void;
  style?: React.CSSProperties;
}) {
  const menuRef = useRef<HTMLDivElement>(null);

  // Handle click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [onClose]);

  // Generate stable keys for rendering
  let separatorCount = 0;

  return (
    <div
      ref={menuRef}
      style={style}
      className="fixed z-[100] min-w-48 overflow-hidden rounded-md border border-zinc-700 bg-zinc-800 py-1 shadow-xl"
    >
      {items.map((entry) => {
        if ("separator" in entry && entry.separator) {
          const key = `separator-${separatorCount++}`;
          return <div key={key} className="my-1 h-px bg-zinc-700" aria-hidden="true" />;
        }

        const menuItem = entry as MenuItemDef;
        return <MenuItem key={menuItem.id} item={menuItem} onClose={onClose} />;
      })}
    </div>
  );
}

// ============================================================================
// Provider Component
// ============================================================================

export function ContextMenuProvider({ children }: { children: ReactNode }) {
  const [menu, setMenu] = useState<ContextMenuState | null>(null);

  const show = useCallback((x: number, y: number, items: MenuEntry[]) => {
    // Adjust position to keep menu in viewport
    const menuWidth = 200;
    const menuHeight = items.length * 32 + 16;

    const adjustedX = Math.min(x, window.innerWidth - menuWidth - 10);
    const adjustedY = Math.min(y, window.innerHeight - menuHeight - 10);

    setMenu({ x: Math.max(10, adjustedX), y: Math.max(10, adjustedY), items });
  }, []);

  const hide = useCallback(() => {
    setMenu(null);
  }, []);

  return (
    <ContextMenuContext.Provider value={{ show, hide }}>
      {children}
      {menu && <Menu items={menu.items} onClose={hide} style={{ left: menu.x, top: menu.y }} />}
    </ContextMenuContext.Provider>
  );
}
