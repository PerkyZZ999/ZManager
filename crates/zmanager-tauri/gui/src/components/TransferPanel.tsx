/**
 * TransferPanel component
 *
 * Shows active file transfer jobs with:
 * - Progress bars with percentages
 * - Pause/resume/cancel controls
 * - Speed and ETA display
 * - Job status indicators
 */

import clsx from "clsx";
import { useState } from "react";
import { SvgIcon } from "./SvgIcon";

// ============================================================================
// Types
// ============================================================================

export type JobStatus = "pending" | "running" | "paused" | "completed" | "failed" | "cancelled";

export interface TransferJob {
  id: string;
  type: "copy" | "move" | "delete";
  sourcePath: string;
  destPath?: string;
  status: JobStatus;
  progress: number; // 0-100
  bytesTransferred: number;
  totalBytes: number;
  speed: number; // bytes per second
  itemsCompleted: number;
  totalItems: number;
  currentFile?: string;
  error?: string;
  startTime: number;
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

function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

function formatEta(bytesRemaining: number, speed: number): string {
  if (speed === 0) return "—";
  const seconds = Math.ceil(bytesRemaining / speed);
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.ceil(seconds / 60)}m`;
  return `${Math.floor(seconds / 3600)}h ${Math.ceil((seconds % 3600) / 60)}m`;
}

function getJobIcon(type: TransferJob["type"]): string {
  switch (type) {
    case "copy":
      return "ic_copy";
    case "move":
      return "ic_move";
    case "delete":
      return "ic_trash";
  }
}

function getStatusColor(status: JobStatus): string {
  switch (status) {
    case "running":
      return "bg-primary";
    case "paused":
      return "bg-yellow-500";
    case "completed":
      return "bg-green-500";
    case "failed":
      return "bg-red-500";
    case "cancelled":
      return "bg-zinc-500";
    default:
      return "bg-zinc-600";
  }
}

// ============================================================================
// Job Item Component
// ============================================================================

interface JobItemProps {
  job: TransferJob;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onDismiss: (id: string) => void;
}

function JobItem({ job, onPause, onResume, onCancel, onDismiss }: JobItemProps) {
  const isActive = job.status === "running" || job.status === "paused";
  const isFinished =
    job.status === "completed" || job.status === "failed" || job.status === "cancelled";
  const bytesRemaining = job.totalBytes - job.bytesTransferred;

  return (
    <div className="rounded-lg bg-zinc-800/50 p-3 ring-1 ring-white/5">
      {/* Header */}
      <div className="mb-2 flex items-center gap-2">
        <SvgIcon name={getJobIcon(job.type)} size={16} />
        <span className="flex-1 truncate font-medium text-sm capitalize">{job.type}</span>
        <span
          className={clsx(
            "rounded px-1.5 py-0.5 font-medium text-xs capitalize",
            getStatusColor(job.status)
          )}
        >
          {job.status}
        </span>
      </div>

      {/* Current file */}
      {job.currentFile && <p className="mb-2 truncate text-xs text-zinc-400">{job.currentFile}</p>}

      {/* Progress bar */}
      <div className="mb-2 h-2 overflow-hidden rounded-full bg-zinc-700">
        <div
          className={clsx("h-full transition-all duration-300", getStatusColor(job.status))}
          style={{ width: `${job.progress}%` }}
        />
      </div>

      {/* Stats row */}
      <div className="mb-2 flex items-center justify-between text-xs text-zinc-400">
        <span>
          {formatBytes(job.bytesTransferred)} / {formatBytes(job.totalBytes)} (
          {job.progress.toFixed(1)}%)
        </span>
        {isActive && job.speed > 0 && (
          <span>
            {formatSpeed(job.speed)} · ETA {formatEta(bytesRemaining, job.speed)}
          </span>
        )}
      </div>

      {/* Items count */}
      <div className="mb-3 text-xs text-zinc-500">
        {job.itemsCompleted} / {job.totalItems} items
      </div>

      {/* Error message */}
      {job.error && <p className="mb-3 text-red-400 text-xs">{job.error}</p>}

      {/* Controls */}
      <div className="flex items-center gap-2">
        {isActive && (
          <>
            {job.status === "running" ? (
              <button
                type="button"
                onClick={() => onPause(job.id)}
                className="rounded bg-zinc-700 px-3 py-1 text-xs transition-colors hover:bg-zinc-600"
              >
                Pause
              </button>
            ) : (
              <button
                type="button"
                onClick={() => onResume(job.id)}
                className="rounded bg-primary px-3 py-1 text-xs transition-colors hover:bg-primary/80"
              >
                Resume
              </button>
            )}
            <button
              type="button"
              onClick={() => onCancel(job.id)}
              className="rounded bg-red-600/20 px-3 py-1 text-red-400 text-xs transition-colors hover:bg-red-600/30"
            >
              Cancel
            </button>
          </>
        )}
        {isFinished && (
          <button
            type="button"
            onClick={() => onDismiss(job.id)}
            className="rounded bg-zinc-700 px-3 py-1 text-xs transition-colors hover:bg-zinc-600"
          >
            Dismiss
          </button>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Main TransferPanel Component
// ============================================================================

export interface TransferPanelProps {
  jobs: TransferJob[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onDismiss: (id: string) => void;
  onClearCompleted: () => void;
}

export function TransferPanel({
  jobs,
  onPause,
  onResume,
  onCancel,
  onDismiss,
  onClearCompleted,
}: TransferPanelProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const activeJobs = jobs.filter((j) => j.status === "running" || j.status === "paused");
  const completedJobs = jobs.filter(
    (j) => j.status === "completed" || j.status === "failed" || j.status === "cancelled"
  );

  if (jobs.length === 0) {
    return null;
  }

  return (
    <div className="border-zinc-700 border-t bg-zinc-900">
      {/* Header */}
      <button
        type="button"
        onClick={() => setIsExpanded(!isExpanded)}
        className="flex w-full items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-white/5"
      >
        <SvgIcon name={isExpanded ? "ic_chevron_down" : "ic_chevron_right"} size={14} />
        <span className="flex-1 font-medium text-sm">Transfers ({activeJobs.length} active)</span>
        {completedJobs.length > 0 && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onClearCompleted();
            }}
            className="rounded bg-zinc-700 px-2 py-0.5 text-xs transition-colors hover:bg-zinc-600"
          >
            Clear completed
          </button>
        )}
      </button>

      {/* Job list */}
      {isExpanded && (
        <div className="max-h-64 space-y-2 overflow-y-auto px-3 pb-3">
          {jobs.map((job) => (
            <JobItem
              key={job.id}
              job={job}
              onPause={onPause}
              onResume={onResume}
              onCancel={onCancel}
              onDismiss={onDismiss}
            />
          ))}
        </div>
      )}
    </div>
  );
}
