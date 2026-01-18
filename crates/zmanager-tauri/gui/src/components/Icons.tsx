/**
 * Windows-style SVG icons for the file manager.
 *
 * These icons are designed to match Windows 11 File Explorer aesthetic
 * with clean, minimal strokes on a dark background.
 */

import type { SVGProps } from "react";

interface IconProps extends SVGProps<SVGSVGElement> {
  size?: number;
}

/** Base icon wrapper */
function Icon({ size = 16, className = "", ...props }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      {...props}
    />
  );
}

// ============================================================================
// Sidebar Icons
// ============================================================================

export function HomeIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
    </Icon>
  );
}

export function DesktopIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <rect x="2" y="3" width="20" height="14" rx="2" />
      <path d="M8 21h8M12 17v4" />
    </Icon>
  );
}

export function DownloadsIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </Icon>
  );
}

export function DocumentsIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <polyline points="10 9 9 9 8 9" />
    </Icon>
  );
}

// ============================================================================
// Drive Icons
// ============================================================================

export function DriveFixedIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <circle cx="7" cy="16" r="1.5" fill="currentColor" stroke="none" />
      <line x1="11" y1="8" x2="17" y2="8" />
      <line x1="11" y1="12" x2="15" y2="12" />
    </Icon>
  );
}

export function DriveRemovableIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <rect x="4" y="6" width="16" height="12" rx="1" />
      <rect x="7" y="9" width="4" height="6" rx="0.5" />
      <path d="M9 6V4a1 1 0 011-1h4a1 1 0 011 1v2" />
    </Icon>
  );
}

export function DriveNetworkIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <circle cx="12" cy="12" r="10" />
      <ellipse cx="12" cy="12" rx="10" ry="4" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z" />
    </Icon>
  );
}

export function DriveCdRomIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <circle cx="12" cy="12" r="10" />
      <circle cx="12" cy="12" r="3" />
    </Icon>
  );
}

export function DriveRamIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <rect x="2" y="8" width="20" height="8" rx="1" />
      <path d="M6 8V6M10 8V6M14 8V6M18 8V6" />
      <path d="M6 16v2M10 16v2M14 16v2M18 16v2" />
    </Icon>
  );
}

// ============================================================================
// File Type Icons
// ============================================================================

export function FolderIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
    </Icon>
  );
}

export function FolderOpenIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2v2" />
      <path d="M2 10h20l-2 9H4l-2-9z" />
    </Icon>
  );
}

export function FileIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
    </Icon>
  );
}

export function FileTextIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <line x1="10" y1="9" x2="8" y2="9" />
    </Icon>
  );
}

export function FileCodeIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <polyline points="10 13 8 15 10 17" />
      <polyline points="14 13 16 15 14 17" />
    </Icon>
  );
}

export function FileImageIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
      <circle cx="8.5" cy="8.5" r="1.5" />
      <polyline points="21 15 16 10 5 21" />
    </Icon>
  );
}

export function FileAudioIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M9 18V5l12-2v13" />
      <circle cx="6" cy="18" r="3" />
      <circle cx="18" cy="16" r="3" />
    </Icon>
  );
}

export function FileVideoIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polygon points="23 7 16 12 23 17 23 7" />
      <rect x="1" y="5" width="15" height="14" rx="2" ry="2" />
    </Icon>
  );
}

export function FileArchiveIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <path
        d="M10 12h1v1h-1zM12 12h1v1h-1zM10 14h1v1h-1zM12 14h1v1h-1zM10 16h1v1h-1zM12 16h1v1h-1z"
        fill="currentColor"
        stroke="none"
      />
    </Icon>
  );
}

export function FileExecutableIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <path d="M9 9h6v6H9z" />
      <path d="M9 3v6M15 3v6M9 15v6M15 15v6M3 9h6M15 9h6M3 15h6M15 15h6" />
    </Icon>
  );
}

export function FilePdfIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <text x="7" y="17" fontSize="6" fontWeight="bold" fill="currentColor" stroke="none">
        PDF
      </text>
    </Icon>
  );
}

export function SymlinkIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" />
      <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" />
    </Icon>
  );
}

export function JunctionIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <circle cx="12" cy="12" r="10" />
      <path d="M12 8l4 4-4 4M8 12h8" />
    </Icon>
  );
}

// ============================================================================
// Navigation Icons
// ============================================================================

export function ChevronLeftIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polyline points="15 18 9 12 15 6" />
    </Icon>
  );
}

export function ChevronRightIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polyline points="9 18 15 12 9 6" />
    </Icon>
  );
}

export function ChevronUpIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polyline points="18 15 12 9 6 15" />
    </Icon>
  );
}

export function ChevronDownIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polyline points="6 9 12 15 18 9" />
    </Icon>
  );
}

export function RefreshIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polyline points="23 4 23 10 17 10" />
      <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10" />
    </Icon>
  );
}

// ============================================================================
// Section Icons
// ============================================================================

export function StarIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
    </Icon>
  );
}

export function HardDriveIcon(props: IconProps) {
  return (
    <Icon {...props}>
      <line x1="22" y1="12" x2="2" y2="12" />
      <path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z" />
      <line x1="6" y1="16" x2="6.01" y2="16" />
      <line x1="10" y1="16" x2="10.01" y2="16" />
    </Icon>
  );
}

// ============================================================================
// Utility function to get file icon by entry
// ============================================================================

import type { EntryMeta } from "../types";

export function getFileIcon(entry: EntryMeta): React.ComponentType<IconProps> {
  if (entry.kind === "directory") return FolderIcon;
  if (entry.kind === "symlink") return SymlinkIcon;
  if (entry.kind === "junction") return JunctionIcon;

  // File icons based on extension
  switch (entry.extension?.toLowerCase()) {
    case "txt":
    case "md":
    case "log":
    case "ini":
    case "cfg":
      return FileTextIcon;

    case "pdf":
      return FilePdfIcon;

    case "doc":
    case "docx":
    case "odt":
    case "rtf":
      return FileTextIcon;

    case "jpg":
    case "jpeg":
    case "png":
    case "gif":
    case "bmp":
    case "webp":
    case "svg":
    case "ico":
      return FileImageIcon;

    case "mp3":
    case "wav":
    case "flac":
    case "ogg":
    case "m4a":
    case "aac":
      return FileAudioIcon;

    case "mp4":
    case "mkv":
    case "avi":
    case "mov":
    case "wmv":
    case "webm":
      return FileVideoIcon;

    case "zip":
    case "rar":
    case "7z":
    case "tar":
    case "gz":
    case "bz2":
    case "xz":
      return FileArchiveIcon;

    case "exe":
    case "msi":
    case "bat":
    case "cmd":
    case "ps1":
      return FileExecutableIcon;

    case "js":
    case "ts":
    case "jsx":
    case "tsx":
    case "rs":
    case "py":
    case "java":
    case "c":
    case "cpp":
    case "h":
    case "hpp":
    case "cs":
    case "go":
    case "rb":
    case "php":
    case "html":
    case "css":
    case "scss":
    case "less":
    case "json":
    case "yaml":
    case "yml":
    case "toml":
    case "xml":
    case "sql":
    case "sh":
      return FileCodeIcon;

    default:
      return FileIcon;
  }
}

// ============================================================================
// Utility function to get drive icon by type
// ============================================================================

export function getDriveIcon(driveType: string): React.ComponentType<IconProps> {
  switch (driveType) {
    case "Removable":
      return DriveRemovableIcon;
    case "Network":
      return DriveNetworkIcon;
    case "CdRom":
      return DriveCdRomIcon;
    case "RamDisk":
      return DriveRamIcon;
    default:
      return DriveFixedIcon;
  }
}
