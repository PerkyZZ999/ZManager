<!-- File: IPC_Contract.md -->

# ZManager — IPC Contract (Tauri v2 GUI)

## Overview
The GUI communicates with Rust via:
- Commands: request/response using `invoke(cmd, args)` from the frontend. [web:119]
- Events: Rust emits events and the frontend subscribes using `listen(eventName, handler)`. [web:114]

## Command conventions
- Command names use `zmanager_*` prefix.
- All commands return `{ ok: true, data }` or `{ ok: false, error }` (stable shape).
- Errors include:
  - `code` (string)
  - `message` (string)
  - `path` (optional)
  - `details` (optional: OS error info)

## Core commands (initial)
### zmanager_list_dir
Args:
- `path: string`
- `sort: { key: "name"|"ext"|"size"|"mtime"|"type", dir: "asc"|"desc" }`
- `filter?: string`
Returns:
- `entries: Array<{ name, path, isDir, size?, mtime?, ext? }>`
- `stats: { total, dirs, files }`

### zmanager_open
Args:
- `path: string`
Returns:
- `entry` (metadata)

### zmanager_rename
Args:
- `from: string`
- `to: string`
Returns:
- `changed: boolean`

### zmanager_delete
Args:
- `path: string`
- `recursive: boolean`
- `permanent: boolean` (if true, bypass Recycle Bin; default false)
Returns:
- `deletedCount: number`

### zmanager_mkdir
Args:
- `path: string`
Returns:
- `created: boolean`

### zmanager_open_default
Args:
- `path: string`
Returns:
- `opened: boolean`
Notes:
- Opens file with system default application (ShellExecute on Windows).

### zmanager_get_settings
Args: none
Returns:
- `showHidden: boolean`
- `showSystem: boolean`
- `defaultConflictPolicy: "ask"|"overwrite"|"skip"|"rename"`

### zmanager_set_settings
Args:
- `showHidden?: boolean`
- `showSystem?: boolean`
- `defaultConflictPolicy?: "ask"|"overwrite"|"skip"|"rename"`
Returns:
- `updated: boolean`

### zmanager_get_properties
Args:
- `path: string`
Returns:
- `properties: { name, path, isDir, isSymlink, size?, itemCount?, created, modified, accessed, attributes, target? }`

### zmanager_get_favorites
Args: none
Returns:
- `favorites: Array<{ id: string, name: string, path: string, order: number }>`

### zmanager_add_favorite
Args:
- `path: string`
- `name?: string` (optional custom name, defaults to folder name)
Returns:
- `added: { id: string, name: string, path: string }`

### zmanager_remove_favorite
Args:
- `id: string`
Returns:
- `removed: boolean`

### zmanager_reorder_favorites
Args:
- `orderedIds: Array<string>`
Returns:
- `updated: boolean`

### zmanager_autocomplete_path
Args:
- `partial: string` (partial path typed by user)
Returns:
- `suggestions: Array<{ path: string, name: string, isDir: boolean }>`
Notes:
- Returns matching directories for address bar autocomplete.
- Limited to first 20 matches for performance.

### zmanager_clipboard_copy
Args:
- `paths: Array<string>`
- `cut: boolean` (true for cut, false for copy)
Returns:
- `copied: boolean`
Notes:
- Places paths in Windows clipboard in CF_HDROP format.

### zmanager_clipboard_paste
Args:
- `targetDir: string`
Returns:
- `jobId?: string` (if paste initiates a transfer)
- `pasted: boolean`
Notes:
- Reads from Windows clipboard and initiates copy/move.

### zmanager_get_drives
Args: none
Returns:
- `drives: Array<{ letter: string, label?: string, totalBytes: number, freeBytes: number, driveType: "fixed"|"removable"|"network"|"cdrom" }>`
Notes:
- Returns list of available drives on Windows.

### zmanager_get_disk_space
Args:
- `path: string`
Returns:
- `totalBytes: number`
- `freeBytes: number`
- `usedBytes: number`
Notes:
- Returns disk space info for the drive containing the given path.

## Transfer commands
### zmanager_transfer_start
Args:
- `items: Array<{ from: string, toDir: string }>`
- `mode: "copy"|"move"`
- `conflict: "ask"|"overwrite"|"skip"|"rename"`
Returns:
- `jobId: string`

### zmanager_transfer_pause
Args:
- `jobId: string`
Returns:
- `paused: boolean`

### zmanager_transfer_resume
Args:
- `jobId: string`
Returns:
- `running: boolean`

### zmanager_transfer_cancel
Args:
- `jobId: string`
Returns:
- `canceled: boolean`

### zmanager_jobs_list
Args: none
Returns:
- `jobs: Array<{ jobId, kind, state, progress? }>`

### zmanager_job_report
Args:
- `jobId: string`
Returns:
- `report: Array<{ path: string, status: "success"|"failed"|"skipped", error?: { code, message } }>`
- `summary: { total, succeeded, failed, skipped }`

## File Watching commands
### zmanager_watch_start
Args:
- `path: string`
- `watchId: string` (client-provided ID for this watch)
Returns:
- `watching: boolean`

### zmanager_watch_stop
Args:
- `watchId: string`
Returns:
- `stopped: boolean`

## Events (Rust -> Frontend)
Frontend listens using `listen("zmanager://event-name", handler)`. [web:114]

### zmanager://job-progress
Payload:
- `jobId`
- `bytesDone`
- `bytesTotal?`
- `itemsDone`
- `itemsTotal?`
- `currentPath?`
- `throughputBytesPerSec?`

### zmanager://job-state
Payload:
- `jobId`
- `state: "queued"|"running"|"paused"|"completed"|"failed"|"canceled"`
- `error?` (same shape as command errors)
- `report?` (summary on completion: `{ total, succeeded, failed, skipped }`)

### zmanager://dir-changed
Payload:
- `watchId: string`
- `path: string`
- `kind: "create"|"modify"|"delete"|"rename"`
- `affectedPaths: Array<string>` (paths that changed)
Notes:
- Events are debounced (~300ms) to avoid rapid-fire updates.

### zmanager://conflict-ask
Payload:
- `jobId: string`
- `conflictId: string`
- `srcPath: string`
- `dstPath: string`
- `srcMeta: { size, mtime }`
- `dstMeta: { size, mtime }`
Notes:
- Emitted when conflict policy is "ask" and a conflict is encountered.
- Frontend must respond via `zmanager_conflict_resolve` command.

## Commands (Frontend -> Rust, additional)
### zmanager_conflict_resolve
Args:
- `conflictId: string`
- `action: "overwrite"|"skip"|"rename"|"cancel"`
- `applyToAll: boolean`
Returns:
- `accepted: boolean`

## Versioning
- IPC contract version string: `ipcVersion: "1.0"`
- Any breaking change must bump `ipcVersion` and keep an adapter for one minor release when possible.

## Notes
- Windows-native copy uses CopyFileEx for progress callback capabilities; progress is forwarded into zmanager://job-progress. [web:28]
