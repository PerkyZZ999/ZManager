/**
 * Typed Tauri IPC client for ZManager.
 * Wraps `@tauri-apps/api/core` invoke with proper typing.
 */

import { invoke } from "@tauri-apps/api/core";
import type { DirListing, DriveInfo, FilterSpec, IpcResponse, SortSpec } from "../types";

// ============================================================================
// IPC Error Handling
// ============================================================================

/** Custom error class for IPC failures */
export class IpcError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "IpcError";
  }
}

/** Unwrap IPC response, throwing on error */
function unwrap<T>(response: IpcResponse<T>): T {
  if (!response.ok || response.data === undefined) {
    throw new IpcError(response.error ?? "Unknown IPC error");
  }
  return response.data;
}

// ============================================================================
// Directory Operations
// ============================================================================

/**
 * List directory contents with optional sorting and filtering.
 *
 * @param path - Absolute path to the directory
 * @param sort - Optional sorting specification
 * @param filter - Optional filtering specification
 * @returns Directory listing with entries and statistics
 */
export async function listDir(
  path: string,
  sort?: SortSpec,
  filter?: FilterSpec
): Promise<DirListing> {
  const response = await invoke<IpcResponse<DirListing>>("zmanager_list_dir", {
    path,
    sort: sort ?? null,
    filter: filter ?? null,
  });
  return unwrap(response);
}

/**
 * Navigate to a directory and get its contents.
 * Validates that the path exists and is a directory.
 *
 * @param path - Absolute path to navigate to
 * @param sort - Optional sorting specification
 * @param filter - Optional filtering specification
 * @returns Directory listing
 */
export async function navigate(
  path: string,
  sort?: SortSpec,
  filter?: FilterSpec
): Promise<DirListing> {
  const response = await invoke<IpcResponse<DirListing>>("zmanager_navigate", {
    path,
    sort: sort ?? null,
    filter: filter ?? null,
  });
  return unwrap(response);
}

/**
 * Get the parent directory path.
 *
 * @param path - Current path
 * @returns Parent path, or null if at root
 */
export async function getParent(path: string): Promise<string | null> {
  const response = await invoke<IpcResponse<string | null>>("zmanager_get_parent", { path });
  return unwrap(response);
}

// ============================================================================
// Drive Operations
// ============================================================================

/**
 * Get list of available drives on the system.
 *
 * @returns Array of drive information
 */
export async function getDrives(): Promise<DriveInfo[]> {
  const response = await invoke<IpcResponse<DriveInfo[]>>("zmanager_get_drives");
  return unwrap(response);
}

// ============================================================================
// File Operations
// ============================================================================

/** Result of a delete operation */
export interface DeleteResult {
  deleted: number;
  failed: number;
  errors: string[];
}

/**
 * Delete files/folders to the Recycle Bin.
 *
 * @param paths - Array of absolute paths to delete
 * @returns Delete result with counts and any errors
 */
export async function deleteEntries(paths: string[]): Promise<DeleteResult> {
  const response = await invoke<IpcResponse<DeleteResult>>("zmanager_delete_entries", { paths });
  return unwrap(response);
}

/**
 * Rename a file or folder.
 *
 * @param path - Absolute path of the entry to rename
 * @param newName - New name (without path separators)
 * @returns New absolute path after rename
 */
export async function renameEntry(path: string, newName: string): Promise<string> {
  const response = await invoke<IpcResponse<string>>("zmanager_rename_entry", { path, newName });
  return unwrap(response);
}

/**
 * Create a new folder.
 *
 * @param parent - Parent directory path
 * @param name - New folder name
 * @returns Absolute path of the created folder
 */
export async function createFolder(parent: string, name: string): Promise<string> {
  const response = await invoke<IpcResponse<string>>("zmanager_create_folder", { parent, name });
  return unwrap(response);
}

/**
 * Create a new empty file.
 *
 * @param parent - Parent directory path
 * @param name - New file name
 * @returns Absolute path of the created file
 */
export async function createFile(parent: string, name: string): Promise<string> {
  const response = await invoke<IpcResponse<string>>("zmanager_create_file", { parent, name });
  return unwrap(response);
}

/**
 * Open a file or folder with the default application.
 *
 * @param path - Absolute path to open
 */
export async function openFile(path: string): Promise<void> {
  const response = await invoke<IpcResponse<null>>("zmanager_open_file", { path });
  unwrap(response);
}

/** File properties from backend */
export interface FileProperties {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  is_readonly: boolean;
  is_hidden: boolean;
  is_system: boolean;
  created: string | null;
  modified: string | null;
  accessed: string | null;
}

/**
 * Get properties of a file or folder.
 *
 * @param path - Absolute path
 * @returns File properties
 */
export async function getProperties(path: string): Promise<FileProperties> {
  const response = await invoke<IpcResponse<FileProperties>>("zmanager_get_properties", { path });
  return unwrap(response);
}

// ============================================================================
// Re-exports for convenience
// ============================================================================

export type { DirListing, DriveInfo, EntryMeta, FilterSpec, SortSpec } from "../types";
