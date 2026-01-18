/**
 * NewButton component
 *
 * A dropdown button for creating new files and folders,
 * similar to Windows 11 File Explorer's "New" button.
 */

import clsx from "clsx";
import { useCallback, useEffect, useRef, useState } from "react";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

export interface NewItemType {
  id: string;
  label: string;
  icon: string;
  extension?: string;
}

interface NewButtonProps {
  onNewFolder: () => void;
  onNewFile: (name: string) => void;
  className?: string;
}

// ============================================================================
// Default new item types
// ============================================================================

const NEW_ITEM_TYPES: NewItemType[] = [
  {
    id: "folder",
    label: "Folder",
    icon: "ic_folder_add",
  },
  {
    id: "text",
    label: "Text Document",
    icon: "ic_document_add",
    extension: ".txt",
  },
  {
    id: "markdown",
    label: "Markdown File",
    icon: "ic_document",
    extension: ".md",
  },
  {
    id: "json",
    label: "JSON File",
    icon: "ic_braces",
    extension: ".json",
  },
];

// ============================================================================
// NewButton Component
// ============================================================================

export function NewButton({ onNewFolder, onNewFile, className }: NewButtonProps) {
  const [isOpen, setIsOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (
        menuRef.current &&
        !menuRef.current.contains(target) &&
        buttonRef.current &&
        !buttonRef.current.contains(target)
      ) {
        setIsOpen(false);
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setIsOpen(false);
        buttonRef.current?.focus();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen]);

  const handleItemClick = useCallback(
    (item: NewItemType) => {
      setIsOpen(false);
      if (item.id === "folder") {
        onNewFolder();
      } else if (item.extension) {
        onNewFile(`New ${item.label}${item.extension}`);
      }
    },
    [onNewFolder, onNewFile]
  );

  return (
    <div className={clsx("relative", className)}>
      <button
        ref={buttonRef}
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className={clsx(
          "flex items-center gap-1 rounded px-2 py-1 text-sm transition-colors",
          "bg-primary/20 text-primary hover:bg-primary/30",
          isOpen && "bg-primary/30"
        )}
        aria-label="Create new item"
        aria-expanded={isOpen}
        aria-haspopup="menu"
        title="Create new file or folder"
      >
        <SvgIcon name="ic_add" size={14} />
        <span>New</span>
        <SvgIcon
          name="ic_chevron_down"
          size={12}
          className={clsx("transition-transform", isOpen && "rotate-180")}
        />
      </button>

      {/* Dropdown menu */}
      {isOpen && (
        <div
          ref={menuRef}
          role="menu"
          className={clsx(
            "absolute top-full left-0 z-50 mt-1",
            "min-w-48 overflow-hidden rounded-md",
            "border border-zinc-700 bg-zinc-800 py-1 shadow-xl"
          )}
        >
          {NEW_ITEM_TYPES.map((item, index) => (
            <div key={item.id}>
              {/* Separator after folder */}
              {index === 1 && <div className="my-1 h-px bg-zinc-700" aria-hidden="true" />}
              <button
                type="button"
                role="menuitem"
                onClick={() => handleItemClick(item)}
                className={clsx(
                  "flex w-full items-center gap-3 px-3 py-1.5 text-left text-sm",
                  "text-zinc-200 transition-colors hover:bg-white/10"
                )}
              >
                <SvgIcon name={item.icon} size={16} />
                <span>{item.label}</span>
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
