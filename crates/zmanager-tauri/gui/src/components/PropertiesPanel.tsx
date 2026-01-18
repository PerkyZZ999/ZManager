/**
 * PropertiesPanel component
 *
 * Displays detailed information about the selected file(s) or folder(s).
 * Can be shown as a modal or sidebar panel.
 */

import clsx from "clsx";
import { useEffect, useState } from "react";
import type { EntryMeta } from "../types";
import { getIconForEntry } from "../utils/iconMappings";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

interface PropertiesPanelProps {
  entries: EntryMeta[];
  onClose: () => void;
  asModal?: boolean;
}

interface FolderStats {
  totalSize: number;
  fileCount: number;
  folderCount: number;
  isCalculating: boolean;
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
  return `${value.toFixed(2)} ${units[unitIndex]}`;
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return "—";
  try {
    return new Date(dateStr).toLocaleString(undefined, {
      dateStyle: "long",
      timeStyle: "medium",
    });
  } catch {
    return "—";
  }
}

function getFileType(entry: EntryMeta): string {
  if (entry.kind === "directory") return "Folder";
  if (entry.kind === "junction") return "Junction (Symbolic Link)";
  if (entry.kind === "symlink") return "Symbolic Link";
  if (entry.extension) {
    return `${entry.extension.toUpperCase()} File`;
  }
  return "File";
}

// ============================================================================
// Property Row Component
// ============================================================================

function PropertyRow({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
}) {
  return (
    <div className="flex py-1.5">
      <span className="w-28 flex-shrink-0 text-sm text-zinc-400">{label}</span>
      <span className={clsx("flex-1 break-all text-sm text-white", mono && "font-mono text-xs")}>
        {value}
      </span>
    </div>
  );
}

// ============================================================================
// Single Entry Properties
// ============================================================================

function SingleEntryProperties({ entry }: { entry: EntryMeta }) {
  const iconInfo = getIconForEntry(entry);
  const [_folderStats, setFolderStats] = useState<FolderStats | null>(null);

  // TODO: Calculate folder stats via IPC when needed
  useEffect(() => {
    if (entry.kind === "directory") {
      setFolderStats({
        totalSize: 0,
        fileCount: 0,
        folderCount: 0,
        isCalculating: true,
      });
      // Would call: zmanager_calculate_folder_stats(entry.path)
    }
  }, [entry]);

  return (
    <div>
      {/* Header with icon and name */}
      <div className="mb-4 flex items-center gap-3">
        <SvgIcon name={iconInfo.symbolName} size={48} />
        <div className="min-w-0 flex-1">
          <h3 className="truncate font-semibold text-lg">{entry.name}</h3>
          <p className="text-sm text-zinc-400">{getFileType(entry)}</p>
        </div>
      </div>

      {/* Divider */}
      <div className="my-3 h-px bg-zinc-700" />

      {/* Properties */}
      <div className="space-y-0">
        <PropertyRow label="Location" value={entry.path} mono />

        {entry.kind !== "directory" && <PropertyRow label="Size" value={formatBytes(entry.size)} />}

        {entry.kind === "directory" && (
          <PropertyRow label="Size" value={<span className="text-zinc-500">Calculating...</span>} />
        )}

        <PropertyRow label="Created" value={formatDate(entry.created)} />
        <PropertyRow label="Modified" value={formatDate(entry.modified)} />
        <PropertyRow label="Accessed" value={formatDate(entry.accessed)} />

        {/* Divider */}
        <div className="my-3 h-px bg-zinc-700" />

        {/* Attributes */}
        <PropertyRow
          label="Attributes"
          value={
            <div className="flex flex-wrap gap-2">
              {entry.attributes.readonly && (
                <span className="rounded bg-zinc-700 px-2 py-0.5 text-xs">Read-only</span>
              )}
              {entry.attributes.hidden && (
                <span className="rounded bg-zinc-700 px-2 py-0.5 text-xs">Hidden</span>
              )}
              {entry.attributes.system && (
                <span className="rounded bg-zinc-700 px-2 py-0.5 text-xs">System</span>
              )}
              {entry.attributes.archive && (
                <span className="rounded bg-zinc-700 px-2 py-0.5 text-xs">Archive</span>
              )}
              {!entry.attributes.readonly &&
                !entry.attributes.hidden &&
                !entry.attributes.system &&
                !entry.attributes.archive && <span className="text-zinc-500">None</span>}
            </div>
          }
        />

        {/* Link target for symlinks */}
        {entry.link_target && (
          <>
            <div className="my-3 h-px bg-zinc-700" />
            <PropertyRow label="Target" value={entry.link_target} mono />
            {entry.is_broken_link && (
              <PropertyRow
                label="Status"
                value={<span className="text-red-400">Broken link (target not found)</span>}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Multiple Entry Properties
// ============================================================================

function MultipleEntryProperties({ entries }: { entries: EntryMeta[] }) {
  const fileCount = entries.filter((e) => e.kind === "file").length;
  const folderCount = entries.filter((e) => e.kind === "directory").length;
  const totalSize = entries.filter((e) => e.kind === "file").reduce((sum, e) => sum + e.size, 0);

  return (
    <div>
      {/* Header */}
      <div className="mb-4 flex items-center gap-3">
        <SvgIcon name="folder_type_folder" size={48} alt="Multiple items" />
        <div>
          <h3 className="font-semibold text-lg">{entries.length} items selected</h3>
          <p className="text-sm text-zinc-400">
            {fileCount} file{fileCount !== 1 ? "s" : ""}
            {folderCount > 0 && `, ${folderCount} folder${folderCount !== 1 ? "s" : ""}`}
          </p>
        </div>
      </div>

      {/* Divider */}
      <div className="my-3 h-px bg-zinc-700" />

      {/* Summary */}
      <div className="space-y-0">
        <PropertyRow label="Total Size" value={formatBytes(totalSize)} />
        <PropertyRow label="Contents" value={`${fileCount} files, ${folderCount} folders`} />
      </div>

      {/* Divider */}
      <div className="my-3 h-px bg-zinc-700" />

      {/* Item list (scrollable if many items) */}
      <div>
        <p className="mb-2 text-sm text-zinc-400">Selected items:</p>
        <div className="max-h-40 overflow-auto rounded border border-zinc-700 bg-zinc-900">
          {entries.map((entry) => {
            const iconInfo = getIconForEntry(entry);
            return (
              <div key={entry.path} className="flex items-center gap-2 px-2 py-1 text-sm">
                <SvgIcon name={iconInfo.symbolName} size={16} />
                <span className="truncate">{entry.name}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Main Component
// ============================================================================

export function PropertiesPanel({ entries, onClose, asModal = true }: PropertiesPanelProps) {
  if (entries.length === 0) {
    return null;
  }

  const content =
    entries.length === 1 ? (
      <SingleEntryProperties entry={entries[0]} />
    ) : (
      <MultipleEntryProperties entries={entries} />
    );

  if (asModal) {
    return (
      <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/60">
        <div className="w-[420px] overflow-hidden rounded-lg border border-zinc-700 bg-zinc-800 shadow-2xl">
          {/* Header */}
          <div className="flex items-center justify-between border-zinc-700 border-b px-4 py-3">
            <h2 className="font-semibold text-lg">Properties</h2>
            <button
              type="button"
              onClick={onClose}
              className="rounded p-1 transition-colors hover:bg-white/10"
              aria-label="Close"
            >
              <SvgIcon name="ic_dismiss" size={16} />
            </button>
          </div>

          {/* Content */}
          <div className="max-h-[70vh] overflow-auto px-4 py-4">{content}</div>

          {/* Footer */}
          <div className="flex justify-end border-zinc-700 border-t px-4 py-3">
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

  // Sidebar panel version
  return (
    <div className="flex h-full w-72 flex-col border-zinc-700 border-l bg-zinc-800">
      {/* Header */}
      <div className="flex items-center justify-between border-zinc-700 border-b px-3 py-2">
        <h2 className="font-semibold text-sm">Properties</h2>
        <button
          type="button"
          onClick={onClose}
          className="rounded p-1 transition-colors hover:bg-white/10"
          aria-label="Close"
        >
          <SvgIcon name="ic_dismiss" size={14} />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-3">{content}</div>
    </div>
  );
}
