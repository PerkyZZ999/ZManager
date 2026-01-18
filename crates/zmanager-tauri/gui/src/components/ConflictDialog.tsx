/**
 * ConflictDialog component
 *
 * Modal dialog for handling file transfer conflicts.
 * Shows source and destination file info with action options:
 * - Skip: Don't transfer this file
 * - Replace: Overwrite destination
 * - Rename: Keep both with new name
 * - "Apply to all" checkbox for batch operations
 */

import { useEffect, useRef, useState } from "react";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

export type ConflictAction = "skip" | "replace" | "rename" | "cancel";

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  modified: string | null;
  isDirectory: boolean;
}

export interface ConflictInfo {
  source: FileInfo;
  destination: FileInfo;
  newName?: string; // Suggested rename
}

export interface ConflictDialogProps {
  conflict: ConflictInfo;
  remainingConflicts: number;
  onResolve: (action: ConflictAction, applyToAll: boolean, newName?: string) => void;
}

// ============================================================================
// Utility Functions
// ============================================================================

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
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

function formatDate(dateStr: string | null): string {
  if (!dateStr) return "Unknown";
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
    return "Unknown";
  }
}

// ============================================================================
// File Info Card Component
// ============================================================================

interface FileInfoCardProps {
  file: FileInfo;
  label: string;
}

function FileInfoCard({ file, label }: FileInfoCardProps) {
  return (
    <div className="rounded-lg bg-zinc-800/50 p-3 ring-1 ring-white/5">
      <p className="mb-2 font-medium text-xs text-zinc-400 uppercase tracking-wider">{label}</p>
      <div className="flex items-start gap-3">
        <SvgIcon
          name={file.isDirectory ? "folder_type_folder" : "file_type_default"}
          size={32}
          alt={file.isDirectory ? "Folder" : "File"}
        />
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium" title={file.name}>
            {file.name}
          </p>
          <p className="mt-1 text-sm text-zinc-400">
            {file.isDirectory ? "Folder" : formatBytes(file.size)}
          </p>
          <p className="text-sm text-zinc-500">Modified: {formatDate(file.modified)}</p>
          <p className="mt-1 truncate text-xs text-zinc-600" title={file.path}>
            {file.path}
          </p>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Main ConflictDialog Component
// ============================================================================

export function ConflictDialog({ conflict, remainingConflicts, onResolve }: ConflictDialogProps) {
  const [applyToAll, setApplyToAll] = useState(false);
  const [renameValue, setRenameValue] = useState(conflict.newName ?? conflict.source.name);
  const [showRenameInput, setShowRenameInput] = useState(false);
  const renameInputRef = useRef<HTMLInputElement>(null);

  // Focus rename input when it becomes visible
  useEffect(() => {
    if (showRenameInput && renameInputRef.current) {
      renameInputRef.current.focus();
    }
  }, [showRenameInput]);

  const handleAction = (action: ConflictAction) => {
    if (action === "rename") {
      if (!showRenameInput) {
        setShowRenameInput(true);
        return;
      }
      onResolve(action, applyToAll, renameValue);
    } else {
      onResolve(action, applyToAll);
    }
  };

  const handleRenameSubmit = () => {
    if (renameValue.trim() && renameValue !== conflict.destination.name) {
      onResolve("rename", applyToAll, renameValue);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div
        className="w-full max-w-lg rounded-xl bg-zinc-900 p-6 shadow-2xl ring-1 ring-white/10"
        role="dialog"
        aria-modal="true"
        aria-labelledby="conflict-dialog-title"
      >
        {/* Header */}
        <div className="mb-4 flex items-center gap-3">
          <SvgIcon name="ic_warning" size={24} alt="Warning" className="text-yellow-500" />
          <h2 id="conflict-dialog-title" className="font-semibold text-lg">
            File Already Exists
          </h2>
        </div>

        {/* Description */}
        <p className="mb-4 text-sm text-zinc-400">
          A {conflict.destination.isDirectory ? "folder" : "file"} with the same name already exists
          in the destination.
        </p>

        {/* File comparison */}
        <div className="mb-4 grid gap-3 md:grid-cols-2">
          <FileInfoCard file={conflict.source} label="Source" />
          <FileInfoCard file={conflict.destination} label="Destination" />
        </div>

        {/* Rename input */}
        {showRenameInput && (
          <div className="mb-4">
            <label htmlFor="rename-input" className="mb-1 block font-medium text-sm">
              New name
            </label>
            <input
              id="rename-input"
              ref={renameInputRef}
              type="text"
              value={renameValue}
              onChange={(e) => setRenameValue(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleRenameSubmit()}
              className="w-full rounded bg-zinc-800 px-3 py-2 ring-1 ring-white/10 focus:outline-none focus:ring-primary"
            />
          </div>
        )}

        {/* Apply to all checkbox */}
        {remainingConflicts > 0 && (
          <label className="mb-4 flex cursor-pointer items-center gap-2">
            <input
              type="checkbox"
              checked={applyToAll}
              onChange={(e) => setApplyToAll(e.target.checked)}
              className="h-4 w-4 rounded border-zinc-600 bg-zinc-800 text-primary focus:ring-primary"
            />
            <span className="text-sm text-zinc-300">
              Apply to all ({remainingConflicts} remaining)
            </span>
          </label>
        )}

        {/* Action buttons */}
        <div className="flex flex-wrap items-center gap-2">
          {showRenameInput ? (
            <>
              <button
                type="button"
                onClick={() => setShowRenameInput(false)}
                className="rounded bg-zinc-700 px-4 py-2 font-medium text-sm transition-colors hover:bg-zinc-600"
              >
                Back
              </button>
              <button
                type="button"
                onClick={handleRenameSubmit}
                disabled={!renameValue.trim() || renameValue === conflict.destination.name}
                className="rounded bg-primary px-4 py-2 font-medium text-sm transition-colors hover:bg-primary/80 disabled:opacity-50"
              >
                Keep Both
              </button>
            </>
          ) : (
            <>
              <button
                type="button"
                onClick={() => handleAction("skip")}
                className="rounded bg-zinc-700 px-4 py-2 font-medium text-sm transition-colors hover:bg-zinc-600"
              >
                Skip
              </button>
              <button
                type="button"
                onClick={() => handleAction("replace")}
                className="rounded bg-red-600 px-4 py-2 font-medium text-sm transition-colors hover:bg-red-500"
              >
                Replace
              </button>
              <button
                type="button"
                onClick={() => handleAction("rename")}
                className="rounded bg-primary px-4 py-2 font-medium text-sm transition-colors hover:bg-primary/80"
              >
                Keep Both
              </button>
              <button
                type="button"
                onClick={() => handleAction("cancel")}
                className="ml-auto rounded bg-zinc-800 px-4 py-2 font-medium text-sm transition-colors hover:bg-zinc-700"
              >
                Cancel
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
