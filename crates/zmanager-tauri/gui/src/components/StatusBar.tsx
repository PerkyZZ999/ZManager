/**
 * StatusBar component
 *
 * Displays:
 * - Current selection info
 * - Directory statistics
 * - Quick actions
 */

import { useFileSystemStore } from "../stores";

/** Format bytes to human-readable string */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let unitIndex = 0;
  let value = bytes;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}

export function StatusBar() {
  const { activePane, left, right } = useFileSystemStore();
  const pane = activePane === "left" ? left : right;
  const { listing, selectedIndices } = pane;

  const fileCount = listing?.file_count ?? 0;
  const dirCount = listing?.dir_count ?? 0;
  const totalSize = listing?.total_size ?? 0;
  const selectedCount = selectedIndices.size;

  // Calculate selected size
  let selectedSize = 0;
  if (listing && selectedCount > 0) {
    for (const index of selectedIndices) {
      const entry = listing.entries[index];
      if (entry && entry.kind === "file") {
        selectedSize += entry.size;
      }
    }
  }

  return (
    <footer className="flex h-6 items-center justify-between border-zinc-700 border-t bg-zinc-800 px-3 text-xs text-zinc-400">
      {/* Left: Selection info */}
      <div className="flex items-center gap-4">
        {selectedCount > 0 ? (
          <span>
            {selectedCount} selected ({formatBytes(selectedSize)})
          </span>
        ) : (
          <span>
            {fileCount} files, {dirCount} folders
          </span>
        )}
      </div>

      {/* Center: Path breadcrumb (optional future feature) */}
      <div className="flex-1" />

      {/* Right: Total size */}
      <div className="flex items-center gap-4">
        <span>Total: {formatBytes(totalSize)}</span>
        <span className="text-zinc-500">|</span>
        <span className="uppercase">{activePane} pane</span>
      </div>
    </footer>
  );
}
