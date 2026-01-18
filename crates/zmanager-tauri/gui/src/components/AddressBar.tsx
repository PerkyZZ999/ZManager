/**
 * AddressBar component
 *
 * Smart address bar with:
 * - Breadcrumb navigation (click to navigate to segment)
 * - Edit mode on click (type path directly)
 * - Path autocomplete dropdown
 */

import clsx from "clsx";
import { useCallback, useEffect, useRef, useState } from "react";
import { listDir } from "../lib/tauri";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

interface AddressBarProps {
  path: string;
  onNavigate: (path: string) => void;
  className?: string;
}

interface BreadcrumbSegment {
  name: string;
  path: string;
}

interface AutocompleteSuggestion {
  name: string;
  path: string;
  isDirectory: boolean;
}

// ============================================================================
// Utility Functions
// ============================================================================

/** Parse a path into breadcrumb segments */
function parsePath(path: string): BreadcrumbSegment[] {
  // Normalize slashes
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);
  const segments: BreadcrumbSegment[] = [];

  // Handle drive letter (e.g., "C:")
  if (parts.length > 0 && parts[0].endsWith(":")) {
    segments.push({
      name: parts[0],
      path: `${parts[0]}\\`,
    });
    parts.shift();
  }

  // Build remaining segments
  let currentPath = segments.length > 0 ? segments[0].path : "";
  for (const part of parts) {
    currentPath = currentPath ? `${currentPath}${part}\\` : `${part}\\`;
    segments.push({
      name: part,
      path: currentPath.replace(/\\$/, ""), // Remove trailing slash for display
    });
  }

  return segments;
}

/** Get the parent directory of a path */
function getParentDir(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/$/, "");
  const lastSlash = normalized.lastIndexOf("/");
  if (lastSlash <= 0) {
    // Root of drive
    const driveMatch = path.match(/^([A-Za-z]:)/);
    return driveMatch ? `${driveMatch[1]}\\` : path;
  }
  return normalized.substring(0, lastSlash);
}

// ============================================================================
// Breadcrumb Component
// ============================================================================

interface BreadcrumbProps {
  segments: BreadcrumbSegment[];
  onNavigate: (path: string) => void;
}

function Breadcrumbs({ segments, onNavigate }: BreadcrumbProps) {
  return (
    <div className="flex items-center gap-0.5 overflow-hidden">
      {segments.map((segment, index) => (
        <span key={segment.path} className="flex flex-shrink-0 items-center">
          {index > 0 && (
            <SvgIcon
              src="/icons/ui/ic_chevron_right_small.svg"
              size={12}
              alt="separator"
              className="mx-0.5 opacity-50"
            />
          )}
          <button
            type="button"
            onClick={() => onNavigate(segment.path)}
            className={clsx(
              "truncate rounded px-1.5 py-0.5 text-sm transition-colors hover:bg-white/10",
              index === segments.length - 1 ? "font-medium text-white" : "text-zinc-400"
            )}
          >
            {segment.name}
          </button>
        </span>
      ))}
    </div>
  );
}

// ============================================================================
// Autocomplete Dropdown Component
// ============================================================================

interface AutocompleteDropdownProps {
  suggestions: AutocompleteSuggestion[];
  selectedIndex: number;
  onSelect: (suggestion: AutocompleteSuggestion) => void;
  visible: boolean;
}

function AutocompleteDropdown({
  suggestions,
  selectedIndex,
  onSelect,
  visible,
}: AutocompleteDropdownProps) {
  if (!visible || suggestions.length === 0) return null;

  return (
    <div className="absolute top-full right-0 left-0 z-50 mt-1 max-h-64 overflow-auto rounded-md border border-zinc-700 bg-zinc-800 shadow-lg">
      {suggestions.map((suggestion, index) => (
        <button
          key={suggestion.path}
          type="button"
          onClick={() => onSelect(suggestion)}
          className={clsx(
            "flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm",
            index === selectedIndex ? "bg-primary/20 text-white" : "hover:bg-white/10"
          )}
        >
          <SvgIcon
            src={
              suggestion.isDirectory
                ? "/icons/filetypes/folder_type_folder.svg"
                : "/icons/filetypes/file_type_default.svg"
            }
            size={16}
            alt={suggestion.isDirectory ? "folder" : "file"}
            invert={false}
          />
          <span className="truncate">{suggestion.name}</span>
        </button>
      ))}
    </div>
  );
}

// ============================================================================
// Main AddressBar Component
// ============================================================================

export function AddressBar({ path, onNavigate, className }: AddressBarProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(path);
  const [suggestions, setSuggestions] = useState<AutocompleteSuggestion[]>([]);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(-1);
  const [showSuggestions, setShowSuggestions] = useState(false);

  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const segments = parsePath(path);

  // Update edit value when path changes
  useEffect(() => {
    if (!isEditing) {
      setEditValue(path);
    }
  }, [path, isEditing]);

  // Focus input when entering edit mode
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  // Fetch autocomplete suggestions
  const fetchSuggestions = useCallback(async (inputPath: string) => {
    if (!inputPath.trim()) {
      setSuggestions([]);
      return;
    }

    try {
      // Get parent directory to list its contents
      const parentDir = getParentDir(inputPath);
      const searchTerm = inputPath.replace(/\\/g, "/").split("/").pop()?.toLowerCase() ?? "";

      const result = await listDir(parentDir);
      const filtered = result.entries
        .filter((entry) => entry.name.toLowerCase().startsWith(searchTerm))
        .slice(0, 10)
        .map((entry) => ({
          name: entry.name,
          path: entry.path,
          isDirectory: entry.kind === "directory" || entry.kind === "junction",
        }));

      setSuggestions(filtered);
      setSelectedSuggestionIndex(-1);
    } catch {
      setSuggestions([]);
    }
  }, []);

  // Debounced autocomplete
  useEffect(() => {
    if (!isEditing || !showSuggestions) return;

    const timer = setTimeout(() => {
      fetchSuggestions(editValue);
    }, 150);

    return () => clearTimeout(timer);
  }, [editValue, isEditing, showSuggestions, fetchSuggestions]);

  // Handle click outside to close edit mode
  useEffect(() => {
    if (!isEditing) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsEditing(false);
        setShowSuggestions(false);
        setEditValue(path);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isEditing, path]);

  const handleSubmit = useCallback(() => {
    const trimmed = editValue.trim();
    if (trimmed && trimmed !== path) {
      onNavigate(trimmed);
    }
    setIsEditing(false);
    setShowSuggestions(false);
  }, [editValue, path, onNavigate]);

  const handleSelectSuggestion = useCallback(
    (suggestion: AutocompleteSuggestion) => {
      if (suggestion.isDirectory) {
        onNavigate(suggestion.path);
      } else {
        setEditValue(suggestion.path);
      }
      setIsEditing(false);
      setShowSuggestions(false);
    },
    [onNavigate]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "Enter":
          e.preventDefault();
          if (selectedSuggestionIndex >= 0 && suggestions[selectedSuggestionIndex]) {
            handleSelectSuggestion(suggestions[selectedSuggestionIndex]);
          } else {
            handleSubmit();
          }
          break;
        case "Escape":
          e.preventDefault();
          setIsEditing(false);
          setShowSuggestions(false);
          setEditValue(path);
          break;
        case "ArrowDown":
          e.preventDefault();
          if (suggestions.length > 0) {
            setSelectedSuggestionIndex((prev) => (prev < suggestions.length - 1 ? prev + 1 : prev));
          }
          break;
        case "ArrowUp":
          e.preventDefault();
          if (suggestions.length > 0) {
            setSelectedSuggestionIndex((prev) => (prev > 0 ? prev - 1 : -1));
          }
          break;
        case "Tab":
          if (suggestions.length > 0 && selectedSuggestionIndex >= 0) {
            e.preventDefault();
            const selected = suggestions[selectedSuggestionIndex];
            if (selected.isDirectory) {
              setEditValue(`${selected.path}\\`);
              fetchSuggestions(`${selected.path}\\`);
            }
          }
          break;
      }
    },
    [
      selectedSuggestionIndex,
      suggestions,
      handleSelectSuggestion,
      handleSubmit,
      path,
      fetchSuggestions,
    ]
  );

  return (
    <div ref={containerRef} className={clsx("relative flex-1 rounded bg-zinc-900", className)}>
      {isEditing ? (
        <>
          <input
            ref={inputRef}
            type="text"
            value={editValue}
            onChange={(e) => {
              setEditValue(e.target.value);
              setShowSuggestions(true);
            }}
            onKeyDown={handleKeyDown}
            onFocus={() => setShowSuggestions(true)}
            className="w-full rounded bg-zinc-900 px-2 py-1 text-sm text-white outline-none ring-1 ring-primary/50"
            placeholder="Enter path..."
            spellCheck={false}
            autoComplete="off"
          />
          <AutocompleteDropdown
            suggestions={suggestions}
            selectedIndex={selectedSuggestionIndex}
            onSelect={handleSelectSuggestion}
            visible={showSuggestions}
          />
        </>
      ) : (
        <button
          type="button"
          onClick={() => setIsEditing(true)}
          className="flex w-full items-center rounded px-2 py-1 text-left transition-colors hover:bg-white/5"
        >
          <Breadcrumbs segments={segments} onNavigate={onNavigate} />
        </button>
      )}
    </div>
  );
}
