/**
 * TypeScript types matching the Rust backend structures.
 * These types mirror the zmanager-core domain types.
 */

// ============================================================================
// IPC Response Wrapper
// ============================================================================

/** Standard IPC response wrapper following { ok, data?, error? } pattern */
export interface IpcResponse<T> {
  ok: boolean;
  data?: T;
  error?: string;
}

// ============================================================================
// Entry Types
// ============================================================================

/** File system entry kind (snake_case to match Rust serde) */
export type EntryKind = "file" | "directory" | "symlink" | "junction";

/** File system entry attributes (Windows-specific) */
export interface EntryAttributes {
  hidden: boolean;
  system: boolean;
  readonly: boolean;
  archive: boolean;
}

/** Metadata for a single file system entry */
export interface EntryMeta {
  /** The file/folder name (not the full path) */
  name: string;
  /** The absolute path to this entry */
  path: string;
  /** The kind of entry */
  kind: EntryKind;
  /** Size in bytes. For directories, typically 0 */
  size: number;
  /** Creation time (ISO 8601) */
  created: string | null;
  /** Last modification time (ISO 8601) */
  modified: string | null;
  /** Last access time (ISO 8601) */
  accessed: string | null;
  /** File attributes */
  attributes: EntryAttributes;
  /** For symlinks/junctions: the resolved target path */
  link_target: string | null;
  /** True if this is a broken symlink */
  is_broken_link: boolean;
  /** The file extension (lowercase, without dot) */
  extension: string | null;
}

/** Directory listing result */
export interface DirListing {
  /** The directory path that was listed */
  path: string;
  /** The entries in this directory */
  entries: EntryMeta[];
  /** Number of files (direct children only) */
  file_count: number;
  /** Number of directories (direct children only) */
  dir_count: number;
  /** Total size of all files (direct children only) */
  total_size: number;
}

// ============================================================================
// Sort & Filter Types
// ============================================================================

/** Sort field options (snake_case to match Rust serde) */
export type SortField = "name" | "size" | "modified" | "created" | "extension" | "kind";

/** Sort order (snake_case to match Rust serde rename_all) */
export type SortOrder = "ascending" | "descending";

/** Sorting specification */
export interface SortSpec {
  field: SortField;
  order: SortOrder;
  directories_first: boolean;
}

/** Kind filter options (snake_case to match Rust serde) */
export type KindFilter = "all" | "files_only" | "directories_only";

/** Filter specification - matches Rust FilterSpec from zmanager-core */
export interface FilterSpec {
  /** Text pattern to match against entry names (case-insensitive) */
  pattern: string | null;
  /** Whether to show hidden files */
  show_hidden: boolean;
  /** Whether to show system files */
  show_system: boolean;
  /** Filter by file extensions (lowercase, without dots) */
  extensions: string[];
  /** Minimum file size in bytes */
  min_size: number | null;
  /** Maximum file size in bytes */
  max_size: number | null;
}

// ============================================================================
// Drive Types
// ============================================================================

/** Drive type */
export type DriveType =
  | "Unknown"
  | "NoRootDir"
  | "Removable"
  | "Fixed"
  | "Network"
  | "CdRom"
  | "RamDisk";

/** Drive information */
export interface DriveInfo {
  /** The drive letter or mount point (e.g., "C:\\") */
  path: string;
  /** Volume label (e.g., "Windows", "Data") */
  label: string;
  /** Total capacity in bytes */
  total_bytes: number | null;
  /** Free space in bytes */
  free_bytes: number | null;
  /** Type of drive */
  drive_type: string;
  /** File system (e.g., "NTFS", "FAT32") */
  file_system: string | null;
  /** Whether the drive is ready/accessible */
  is_ready: boolean;
}

// ============================================================================
// Default Values
// ============================================================================

/** Default sort specification (directories first, name ascending) */
export const DEFAULT_SORT: SortSpec = {
  field: "name",
  order: "ascending",
  directories_first: true,
};

/** Default filter specification (show all including hidden, but not system) */
export const DEFAULT_FILTER: FilterSpec = {
  pattern: null,
  show_hidden: true,
  show_system: false,
  extensions: [],
  min_size: null,
  max_size: null,
};
