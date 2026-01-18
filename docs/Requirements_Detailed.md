<!-- File: Requirements_Detailed.md -->

# ZManager — Detailed Requirements (v1)

## 1) Epics
- E1: Core filesystem model + navigation
- E2: Transfer engine (Windows-native)
- E3: Job system + progress/eventing
- E4: TUI frontend (Ratatui)
- E5: GUI frontend (Tauri v2 + React 19 + Bun)
- E6: Observability (tracing, logs, debug tooling)
- E7: Packaging + release
- E8: File watching + live updates
- E9: Dual-pane mode (TUI + GUI)
- E10: Configuration system (TOML)
- E11: Error handling & notifications
- E12: Crash reporting

## 2) Functional requirements

### 2.1 Core (E1)
FR-C1: List directory entries with stable ordering and deterministic sorting.  
FR-C2: Support sorting by name, extension, size, modified time, and type.  
FR-C3: Support filtering (substring) and incremental search (type-to-filter).  
FR-C4: Selection model supports multi-select and range select.  
FR-C5: Path handling supports long paths on Windows (>260 chars) using extended-length path prefix (`\\?\`) where feasible.  
FR-C6: All Core operations must be UI-agnostic and callable from both TUI and GUI.  
FR-C7: Detect and correctly display symlinks, junctions, and hardlinks with visual indicators.  
FR-C8: Support toggling visibility of hidden and system files.  
FR-C9: Open files with system default application (ShellExecute on Windows).
FR-C10: Quick Access / Favorites:
  - User-defined list of bookmarked directories.
  - Stored in TOML config file.
  - Quick-jump via hotkeys (1-9 for first 9 favorites).
FR-C11: Properties view (read-only in v1):
  - File: name, path, size, created/modified/accessed dates, attributes, type.
  - Folder: name, path, item count, total size (calculated on demand).
  - Symlink: target path, link type.
FR-C12: Clipboard integration:
  - Copy/Cut places file paths in Windows clipboard (CF_HDROP format).
  - Paste reads from Windows clipboard and initiates copy/move.
  - Interoperable with Windows Explorer.
FR-C13: Drives/Volumes list:
  - Display available drives (C:, D:, etc.) in sidebar or navigation.
  - Show drive label, total size, free space.
  - Quick-jump to drive root.
FR-C14: Startup behavior:
  - First launch: open to user's home directory (%USERPROFILE%).
  - Subsequent launches: restore last session (last viewed directories in each pane).
  - Configurable: option to always start at home or specific path.
FR-C15: Empty directory state:
  - Display "This folder is empty" message.
  - Maintain drag-drop target for external drops.
FR-C16: Directory access errors:
  - Permission denied: show clear error message with path.
  - Network unavailable: show error with retry option.
  - Path not found: show error and offer to navigate to parent.

### 2.2 File operations (E1)
FR-O1: Rename file/folder (with conflict detection).  
FR-O2: Delete file/folder with two modes:
  - Default delete: send to Recycle Bin (via SHFileOperation or IFileOperation).
  - Permanent delete: bypass Recycle Bin when Shift modifier is held.
  - Recursive delete support for folders.  
FR-O3: Create folder.  
FR-O4: Copy/move delegates to Transfer Engine for execution and progress reporting.  
FR-O5: Symlink operations:
  - Copy: follow link and copy target contents by default.
  - Move: move the link itself, not the target.
  - Delete: delete the link itself, never the target.

### 2.3 Job system (E3)
FR-J1: Operations execute as cancellable jobs with a unique JobId.  
FR-J2: Jobs emit progress updates (bytes, items, current file) and state transitions (queued/running/paused/completed/failed/canceled).  
FR-J3: Jobs support cancellation; Transfer Engine must honor cancel requests when possible (CopyFileEx supports cancellation via its progress routine design). [web:28]  
FR-J4: Jobs may be prioritized (foreground vs background).  
FR-J5: Job results produce structured errors with a stable error code + message.

### 2.4 Transfer Engine (E2)
FR-T1: Implement file copy using CopyFileEx to enable progress callbacks. [web:28]  
FR-T2: Folder copy/move implemented as:
- Enumerate with Walkdir (or equivalent) to build a plan. [web:83]
- Execute file tasks with concurrency limits.
- Aggregate progress as a single logical job.
FR-T3: Conflict resolution:
- overwrite / skip / rename
- “apply to all”
FR-T4: Pause/resume behavior:
- Pause stops scheduling new file tasks; in-flight single-file copies complete or are canceled.
- Resume continues queue execution.
- UX must clearly communicate that "pause" means "stop starting new files," not "freeze mid-byte."
FR-T5: Move operation semantics:
- Same-volume move: atomic rename (fast, no copy).
- Cross-volume move: copy to destination, delete source only on successful copy.
- Partial move: source files only deleted if destination confirmed written.
FR-T6: Transfer report:
- Structured per-file result list: ✅ copied, ❌ failed (with error), ⏭️ skipped.
- Exportable as text/JSON for power users.
- Displayed in UI as summary dialog on job completion.
FR-T7: Verification (optional v1.5):
- checksum verification (fast hash option; design now, implement later).

### 2.5 TUI (E4)
FR-TUI1: Three primary views:
- Browser view (directory listing + status) — supports dual-pane mode.
- Transfers view (queue + per-job progress).
- Transfer report view (per-file results after job completion).
FR-TUI2: Dual-pane mode (required):
- Side-by-side directory panels.
- Operations between panes (copy/move from left to right or vice versa).
- Independent navigation per pane.
- Toggle between single-pane and dual-pane modes.
FR-TUI3: Keyboard-first navigation:
- move selection, open directory, go up, back/forward history
- search/filter prompt
- Tab to switch between panes in dual-pane mode.
FR-TUI4: TUI must remain responsive while transfers run (Tokio runtime driving background tasks). [web:54]
FR-TUI5: Keybindings configurable (config file).
FR-TUI6: Conflict resolution UI:
- Bottom-bar modal prompt when conflict occurs.
- Single-keypress responses: [O]verwrite, [S]kip, [R]ename, [A]ll-overwrite, [N]one-skip, [C]ancel.
- Shows contextual info (which file is newer/larger).
FR-TUI7: Hidden/system file toggle via keybinding.

### 2.6 GUI (E5)
FR-GUI1: Core flows parity with TUI:
- browse, select, rename/delete/mkdir, copy/move with progress UI
FR-GUI2: Dual-pane mode (required):
- Side-by-side directory panels.
- Operations between panes.
- Toggle between single-pane and dual-pane modes.
FR-GUI3: IPC:
- React calls Rust commands via `invoke()` for request/response operations. [web:119]
- Rust streams progress via Tauri events; React subscribes via `listen()`. [web:114]
FR-GUI4: Bun-only toolchain:
- `bun install` for dependencies. [web:105]
- `bun test` for frontend tests. [web:107]
FR-GUI5: Dev/build commands integrated with Tauri hooks (beforeDevCommand/beforeBuildCommand) and must work with Bun scripts. [web:126]
FR-GUI6: Drag & Drop:
- Internal DnD: use dnd-kit for dragging files between panes and within lists.
- External drop: Tauri native `onDragDropEvent` for files dropped from Windows Explorer.
- External drag-out: Tauri `startDrag` API for dragging files to external apps.
FR-GUI7: Context menus:
- Right-click on file/folder shows context menu.
- Actions: Open, Open with..., Cut, Copy, Paste, Rename, Delete, Delete permanently, Properties.
- Multi-select context menu for batch operations.
FR-GUI8: Virtualized rendering:
- Use @tanstack/react-virtual for directory listings.
- Must handle 50k+ entries without performance degradation.
FR-GUI9: Open with system default:
- Double-click or Enter opens file with default application.
- "Open with..." option in context menu.
FR-GUI10: Hidden/system file toggle in UI (toolbar or menu).
FR-GUI11: Transfer report dialog on job completion.
FR-GUI12: Smart address bar:
  - Displays path as clickable breadcrumb segments.
  - Click on segment: navigate to that directory.
  - Click on empty area (not on segment): enter edit mode.
  - Edit mode: full path editable with autocomplete suggestions.
  - Autocomplete: suggests matching subdirectories as user types.
  - Enter confirms navigation; Escape cancels edit mode.
FR-GUI13: Quick Access sidebar:
  - Displays user's favorite/bookmarked directories.
  - Drag to reorder favorites.
  - Right-click to remove from favorites.
  - "Add to Quick Access" action in context menu.
FR-GUI14: Custom SVG icon pack:
  - Dev-oriented icons (code files, configs, etc.).
  - Common file type icons (documents, images, audio, video, archives).
  - Folder icons with state indicators (empty, has contents).
  - Special icons: symlinks, hidden files, system files.
FR-GUI15: Status bar (bottom):
  - Selection count and total size.
  - Free disk space on current drive.
  - Operation progress summary.

### 2.11 Future Features (v1.5+)
FF-1: Tabs within panes (multiple directories per pane).
FF-2: Preview pane (images, text, code with syntax highlighting).
FF-3: Full properties dialog with editable attributes.
FF-4: Bulk rename with patterns/regex.
FF-5: Windows installer (MSI/NSIS) with shell integration.

### 2.7 File Watching (E8)
FR-FW1: Integrate `notify` crate for directory watching (wraps ReadDirectoryChangesW on Windows).
FR-FW2: Watch the currently displayed directory/directories in both panes.
FR-FW3: Debounce rapid filesystem events (configurable, default ~300ms) to avoid UI thrashing.
FR-FW4: On change detected:
- Refresh directory listing incrementally if possible.
- Preserve selection state when feasible.
- Show subtle indicator when directory content has changed.
FR-FW5: Handle watch errors gracefully (e.g., network drives, permissions).

### 2.8 Configuration (E10)
FR-CFG1: TOML-based configuration file.
FR-CFG2: Config file location: `%APPDATA%\ZManager\config.toml` (Windows).
FR-CFG3: Configurable settings:
  - Keybindings (full customization).
  - Default conflict policy.
  - Show hidden/system files default.
  - Quick Access / Favorites list.
  - File watching debounce interval.
  - Theme preference (GUI: light/dark).
FR-CFG4: Config hot-reload (TUI: optional, GUI: on focus).
FR-CFG5: Sensible defaults if config file missing.

### 2.9 Error Handling & Notifications (E11)
FR-ERR1: TUI: Status bar displays last error with timestamp; scrollable error log.
FR-ERR2: GUI: Toast notifications for transient errors (auto-dismiss after 5s).
FR-ERR3: GUI: Error panel/log for persistent error history.
FR-ERR4: All errors include: action attempted, path(s) involved, OS error code/message.

### 2.10 Crash Reporting (E12)
FR-CR1: Panic hook captures crash info and writes to local crash dump file.
FR-CR2: Crash dump includes: stack trace, operation in progress, timestamp, version.
FR-CR3: On startup after crash: detect dump, offer to view/export.
FR-CR4: No automatic upload; user must explicitly share crash report.

## 3) Non-functional requirements
NFR-1: Performance
- UI should not freeze on large directories; operations must be async/job-based.
- Transfers should target near OS-level throughput for common local-copy scenarios.

NFR-2: Reliability
- No silent failures; errors must include path + action + OS error details.
- Cancel must leave filesystem in a consistent state (even if partially completed).

NFR-3: Observability
- Use tracing spans for each operation and job lifecycle. [web:75]
- Provide a debug mode that increases verbosity and optionally writes logs to disk.

NFR-4: Security (desktop)
- Tauri command surface must be minimized (only expose required commands). [web:52]

## 4) Out of scope (explicit)
- macOS support
- cloud features
- plugin system (defer to v2+ unless required earlier)
- full shell integration (file associations, shell extensions)
- built-in archive management (zip/7z creation/extraction)
