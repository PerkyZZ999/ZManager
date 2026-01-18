<!-- File: TASK_BREAKDOWN.md -->

# ZManager — Task Breakdown (v1 → v2)

This breakdown is organized by the roadmap milestones you defined, and each milestone is split into epics and implementable tickets.

---

## Milestone 1 — ZManager Core

### Epic C0 — Repo/Workspace foundation
- [ ] C0-T1 Create Rust workspace layout:
  - crates: `zmanager-core`, `zmanager-transfer-win`, `zmanager-tui`, `zmanager-gui-tauri`
  - shared crates (optional): `zmanager-types`, `zmanager-protocol`, `zmanager-config`
  - Done when: `cargo test` runs across workspace; CI builds on Windows.

- [ ] C0-T2 Establish code quality gates:
  - formatting + lint + tests in CI
  - Done when: CI fails on formatting/lint errors and runs unit tests.

- [ ] C0-T3 Define error model + result types:
  - `ZError { code, message, path?, source? }`
  - Done when: all core APIs return consistent errors.

### Epic C1 — Domain model (filesystem + UI-agnostic state)
- [ ] C1-T1 Implement core types:
  - `EntryId`, `EntryKind`, `EntryMeta`, `DirListing`, `SortSpec`, `FilterSpec`
  - Done when: unit tests cover sorting/filter serialization and basic invariants.

- [ ] C1-T2 Selection model:
  - single, multi-select, range select
  - Done when: selection tests cover add/remove/toggle/range behavior.

- [ ] C1-T3 Navigation/session state:
  - `current_dir`, back/forward stack, optional tabs scaffolding
  - Done when: navigation can be driven purely by core commands without UI.

### Epic C2 — Directory listing + metadata cache
- [ ] C2-T1 Implement directory list pipeline (sync baseline):
  - read entries + basic metadata
  - Done when: listing works for normal and large folders without panics.

- [ ] C2-T2 Sorting + filtering + incremental search:
  - sorting by name/ext/size/mtime/type
  - Done when: deterministic output given same inputs.

- [ ] C2-T3 Metadata caching strategy:
  - cache keyed by path or file id (as feasible)
  - Done when: repeated renders don’t re-stat everything unnecessarily.

- [ ] C2-T4 Benchmarks (baseline):
  - listing large folder
  - sorting cost
  - Done when: benchmarks produce stable numbers and are checked into repo.

### Epic C3 — Core operations API (non-transfer)
- [ ] C3-T1 Rename
- [ ] C3-T2 Delete:
  - Default: send to Recycle Bin via SHFileOperation/IFileOperation
  - Permanent: bypass Recycle Bin when requested
  - Recursive folder delete support
- [ ] C3-T3 Mkdir
- [ ] C3-T4 "Open/enter directory" command
- [ ] C3-T5 "Open with default app" command (ShellExecute on Windows)
- [ ] C3-T6 Symlink/junction detection and metadata
- [ ] C3-T7 Hidden/system file visibility toggle
- [ ] C3-T8 Properties view data collection:
  - File: size, dates, attributes
  - Folder: item count, total size (async calculation)
  - Symlink: target path resolution
- [ ] C3-T9 Drives/volumes enumeration:
  - List available drives with labels
  - Get total/free space per drive
- [ ] C3-T10 Long path support:
  - Detect paths >240 chars
  - Use extended-length prefix (\\\\?\\) where needed
  - Test with CopyFileEx and other Win32 APIs
- Done when: operations return structured errors and integrate with tracing.

### Epic C4 — Job system (shared by TUI/GUI)
- [ ] C4-T1 Job model:
  - `JobId`, `JobKind`, `JobState`, `Progress { bytesDone/bytesTotal/itemsDone/itemsTotal/currentPath }`
  - Done when: job lifecycle states are fully covered by tests.

- [ ] C4-T2 Job scheduler (Tokio-based):
  - queueing + concurrency limits
  - Done when: multiple jobs can run and UI can poll/subscribe to progress.

- [ ] C4-T3 Cancellation + pause/resume primitives:
  - cooperative cancellation tokens + pause gates
  - Done when: jobs stop starting new work when paused/canceled.

- [ ] C4-T4 Progress event stream API:
  - in-process broadcast channel for frontends to subscribe
  - Done when: both TUI and GUI can consume the same progress feed.

### Epic C5 — File Watching
- [ ] C5-T1 Integrate `notify` crate for directory watching.
- [ ] C5-T2 Implement debouncing layer (configurable, default ~300ms).
- [ ] C5-T3 Watch manager:
  - Track active watches by ID.
  - Handle watch start/stop lifecycle.
- [ ] C5-T4 Event translation:
  - Map notify events to ZManager change events.
  - Handle rename as single event (not delete + create).
- [ ] C5-T5 Error handling:
  - Graceful handling of network drives, permission issues.
  - Fallback to manual refresh if watching fails.
- Done when: UI auto-refreshes when files change externally.

### Epic C6 — Quick Access / Favorites
- [ ] C6-T1 Favorites data model:
  - `Favorite { id, name, path, order }`
  - Stored in config.toml
- [ ] C6-T2 CRUD operations:
  - Add, remove, rename, reorder favorites
- [ ] C6-T3 Validation:
  - Mark broken favorites (path no longer exists)
- Done when: favorites persist across sessions and are editable.

### Epic C7 — Clipboard Integration (Windows)
- [ ] C7-T1 Implement CF_HDROP clipboard write:
  - Copy/Cut places file paths in Windows clipboard
- [ ] C7-T2 Implement CF_HDROP clipboard read:
  - Detect clipboard content type
  - Parse file list from clipboard
- [ ] C7-T3 Preferred drop effect:
  - Track cut vs copy state for paste behavior
- [ ] C7-T4 Paste operation:
  - Initiate copy/move job from clipboard contents
- Done when: copy/paste works between ZManager and Windows Explorer.

### Epic C8 — Configuration System
- [ ] C8-T1 TOML config schema definition:
  - All configurable settings with types
- [ ] C8-T2 Config file loading:
  - Load from %APPDATA%\ZManager\config.toml
  - Create default if missing
- [ ] C8-T3 Config validation:
  - Validate keybindings, paths, values
- [ ] C8-T4 Config hot-reload (optional):
  - Watch config file for changes
- [ ] C8-T5 Config API:
  - Get/set individual settings
  - Save to disk- [ ] C8-T6 Session state persistence:
  - Save last viewed directories (per pane)
  - Restore on startup
  - Configurable: always home vs restore session- Done when: settings persist and can be edited externally.

### Epic C9 — Crash Reporting
- [ ] C9-T1 Panic hook setup:
  - Capture panic info, backtrace
- [ ] C9-T2 Crash dump writer:
  - Write structured crash report to file
  - Include: version, timestamp, stack trace, last operation
- [ ] C9-T3 Crash detection on startup:
  - Detect previous crash dump
  - Offer to view/export
- [ ] C9-T4 Crash report format:
  - Human-readable text + JSON for parsing
- Done when: crashes are captured and user can share reports.

### Epic C10 — Observability (Core)
- [ ] C10-T1 Integrate `tracing` spans around all core commands and jobs.
- [ ] C10-T2 Add `tracing-subscriber` config:
  - debug mode verbosity, file logging toggle
- Done when: a single “trace id” (span context) follows an operation through core → transfer → UI adapters.

---

## Milestone 2 — ZManager File Transfer (Native & Fast, Windows v1)

### Epic T0 — Transfer architecture
- [ ] T0-T1 Define transfer plan model:
  - `TransferItem { src, dst }`
  - `TransferMode: Copy | Move`
  - `ConflictPolicy: Ask | Overwrite | Skip | Rename`
- [ ] T0-T2 Define TransferJob contract:
  - input: items + policy
  - output: per-file result list + aggregated stats
- Done when: core can start a transfer job without knowing implementation details.

### Epic T1 — Windows CopyFileEx integration
- [ ] T1-T1 Implement single-file copy using `CopyFileEx` through `windows`/`windows-sys`. [web:28]
- [ ] T1-T2 Progress callback wiring:
  - map callback progress to `Progress` events
- [ ] T1-T3 Cancellation behavior:
  - if user cancels, return `PROGRESS_CANCEL` so CopyFileEx aborts and deletes the partial destination file. [web:28]
- Done when: large file copy shows continuous progress and can cancel reliably.

### Epic T2 — Folder/tree transfer
- [ ] T2-T1 Build file plan with `walkdir`:
  - enumerate src tree
  - produce destination mapping
- [ ] T2-T2 Execute plan with concurrency limits:
  - schedule N parallel file copies (tunable)
  - keep directory creation ordering correct
- [ ] T2-T3 Aggregate progress:
  - compute totals (items, bytes) before execution when possible
  - stream overall progress in addition to per-file
- Done when: folder copies are correct, fast, cancellable, and observable.

### Epic T3 — Conflict resolution UX hooks
- [ ] T3-T1 Implement overwrite/skip/rename rules
- [ ] T3-T2 “Ask” mode protocol (UI roundtrip):
  - core pauses job on conflict and requests UI decision
  - emit conflict event with metadata (src/dst size, mtime)
  - wait for resolution response with optional "apply to all"
- Done when: behavior is deterministic and matches selected policy.

### Epic T4 — Transfer reliability hardening
- [ ] T4-T1 Locked files / permission denied strategy:
  - mark file failed; continue job; aggregate error
- [ ] T4-T2 Partial failure policy:
  - job ends as failed-with-errors but reports successes
- [ ] T4-T3 Retry policy hooks (off by default):
  - optional exponential backoff for transient errors
- Done when: transfers behave predictably under real-world Windows edge cases.

### Epic T5 — Transfer reporting
- [ ] T5-T1 Implement transfer report model:
  - per-file result: success / failed / skipped
  - error details for failed items
- [ ] T5-T2 Report persistence:
  - store report in memory during job
  - make available via API after job completion
- [ ] T5-T3 Report export:
  - JSON and plain text formats
- Done when: users can review and export detailed transfer results.

---

## Milestone 3 — ZManager TUI (v1)

### Epic TU0 — TUI app scaffolding
- [ ] TU0-T1 Ratatui layout skeleton + state machine
- [ ] TU0-T2 Crossterm event loop integration (keys, resize)
- [ ] TU0-T3 Tokio runtime wiring for background jobs
- Done when: app starts instantly and renders a basic pane layout.

### Epic TU1 — Browser view
- [ ] TU1-T1 Directory list component:
  - render entries, highlight selection, show metadata columns
  - symlink/junction indicator display
- [ ] TU1-T2 Sorting/filter/search UI:
  - filter prompt and incremental search
- [ ] TU1-T3 Navigation commands:
  - enter dir, go up, back/forward history
- [ ] TU1-T4 Hidden/system file toggle (keybinding)
- [ ] TU1-T5 Open file with default app (Enter on file)
- [ ] TU1-T6 Status bar:
  - current path, selection count, free space
  - error messages with timestamp
  - operation feedback (e.g., "3 files copied")
- [ ] TU1-T7 Properties panel (toggle):
  - read-only file/folder properties display
- Done when: browsing is daily-usable with keyboard only.

### Epic TU2 — Dual-pane mode
- [ ] TU2-T1 Dual-pane layout:
  - side-by-side directory panels
  - Tab to switch active pane
- [ ] TU2-T2 Independent navigation per pane:
  - each pane has own path, history, selection
- [ ] TU2-T3 Cross-pane operations:
  - copy/move from active pane to other pane
- [ ] TU2-T4 Single/dual pane toggle (keybinding)
- Done when: power users can efficiently manage files across two locations.

### Epic TU3 — Transfers view
- [ ] TU3-T1 Transfers queue screen
- [ ] TU3-T2 Per-job progress + throughput display
- [ ] TU3-T3 Controls: pause/resume/cancel
- [ ] TU3-T4 Transfer report view after job completion
- Done when: long transfers can be managed fully in TUI.

### Epic TU4 — Conflict resolution UI
- [ ] TU4-T1 Bottom-bar modal prompt:
  - display conflict info (file names, sizes, dates)
- [ ] TU4-T2 Single-keypress responses:
  - [O]verwrite, [S]kip, [R]ename, [A]ll-overwrite, [N]one-skip, [C]ancel
- [ ] TU4-T3 "Apply to all" behavior for batch conflicts
- Done when: conflicts are resolved smoothly without leaving transfer view.

### Epic TU5 — Quick Access sidebar
- [ ] TU5-T1 Favorites panel:
  - display bookmarked directories
  - quick-jump with number keys (1-9)
- [ ] TU5-T2 Add/remove favorites:
  - keybinding to add current directory
  - keybinding to remove selected favorite
- [ ] TU5-T3 Favorites navigation:
  - arrow keys to select, Enter to navigate
- Done when: power users can quickly jump to favorite locations.

### Epic TU6 — Clipboard operations
- [ ] TU6-T1 Copy/Cut commands:
  - place paths in Windows clipboard
- [ ] TU6-T2 Paste command:
  - read clipboard, initiate transfer
- [ ] TU6-T3 Visual feedback:
  - indicate cut files (dimmed)
- Done when: copy/paste works with Windows Explorer.

### Epic TU7 — Config + keymap
- [ ] TU7-T1 Keymap schema + defaults
- [ ] TU7-T2 Config file loading + hot reload (optional)
- Done when: keybindings can be customized without recompiling.

### Epic TU8 — TUI testing + performance checks
- [ ] TU8-T1 Core-driven TUI integration tests (where feasible)
- [ ] TU8-T2 Stress test scenarios (large dirs, long transfers)
- Done when: no major rendering stalls and no panics in stress runs.

---

## Milestone 4 — ZManager GUI (Tauri v2 + React 19 + Bun)

### Epic GU0 — GUI scaffolding (Bun mandatory)
- [ ] GU0-T1 Create Tauri v2 + React project
- [ ] GU0-T2 Enforce Bun workflows:
  - install: `bun install` [web:105]
  - test: `bun test` [web:107]
- [ ] GU0-T3 Define frontend folder structure:
  - app shell, routes/views, shared components, IPC client module
- Done when: `bun install`, `bun test`, and Tauri dev run successfully on Windows.

### Epic GU1 — IPC integration layer
- [ ] GU1-T1 Rust commands surface (Tauri):
  - expose list/rename/delete/mkdir/transfer start
  - expose settings get/set commands
  - expose file watch start/stop commands
- [ ] GU1-T2 Frontend invoke client:
  - typed wrapper over `invoke()` to call Rust commands. [web:128]
- [ ] GU1-T3 Progress event subscription:
  - subscribe via `listen()` to job progress/state events. [web:115]
- [ ] GU1-T4 Directory change event subscription:
  - subscribe to `zmanager://dir-changed` for live updates.
- Done when: GUI can browse a directory and start a transfer with live progress updates.

### Epic GU2 — GUI browsing UX
- [ ] GU2-T1 File list UI with @tanstack/react-virtual:
  - virtualized for 50k+ entries
  - symlink/junction visual indicators
- [ ] GU2-T2 Sorting/filter/search controls
- [ ] GU2-T3 Smart address bar:
  - clickable breadcrumb segments
  - click on segment → navigate to that directory
  - click on empty area → enter edit mode
  - edit mode: full path editable with autocomplete
  - autocomplete suggestions as user types
  - Enter confirms, Escape cancels
- [ ] GU2-T4 Back/forward navigation buttons
- [ ] GU2-T5 Hidden/system file toggle (toolbar button)
- [ ] GU2-T6 Double-click / Enter opens file with default app
- [ ] GU2-T7 Properties panel (sidebar or modal):
  - read-only file/folder properties
- Done when: browsing parity with TUI core flows is achieved.

### Epic GU3 — Dual-pane mode (GUI)
- [ ] GU3-T1 Dual-pane layout component:
  - resizable split view
- [ ] GU3-T2 Independent state per pane:
  - separate path, selection, history
- [ ] GU3-T3 Cross-pane operations:
  - drag or copy/move to other pane
- [ ] GU3-T4 Single/dual pane toggle (toolbar/keybinding)
- Done when: GUI dual-pane matches TUI functionality.

### Epic GU4 — Drag & Drop
- [ ] GU4-T1 Internal DnD with dnd-kit:
  - drag files within list to reorder (if applicable)
  - drag between panes for copy/move
- [ ] GU4-T2 External drop (Tauri native):
  - handle files dropped from Windows Explorer
  - initiate copy/move job based on drop target
- [ ] GU4-T3 External drag-out (Tauri startDrag):
  - drag files from ZManager to external apps
- [ ] GU4-T4 Visual feedback:
  - drop zone highlighting
  - drag preview
- Done when: DnD feels native and intuitive.

### Epic GU5 — Context menus
- [ ] GU5-T1 Context menu component:
  - right-click on file/folder shows menu
- [ ] GU5-T2 Single-item actions:
  - Open, Open with..., Cut, Copy, Paste, Rename, Delete, Delete permanently, Properties
- [ ] GU5-T3 Multi-select actions:
  - batch operations on selection
- [ ] GU5-T4 Background context menu:
  - right-click on empty space: New folder, Paste, Refresh, Properties
- Done when: context menus provide expected Windows-like UX.

### Epic GU6 — Transfer UX
- [ ] GU6-T1 Transfer queue panel/window
- [ ] GU6-T2 Conflict dialog + "apply to all"
- [ ] GU6-T3 Toast notifications:
  - operation completion/failure
  - auto-dismiss after 5s
  - click to view details
- [ ] GU6-T4 Transfer report dialog on completion
- Done when: transfers are controllable and understandable by non-TUI users.

### Epic GU7 — Quick Access sidebar
- [ ] GU7-T1 Favorites sidebar component:
  - display bookmarked directories
  - collapsible panel
- [ ] GU7-T2 Add to favorites:
  - context menu action
  - drag folder to sidebar
- [ ] GU7-T3 Manage favorites:
  - drag to reorder
  - right-click to remove/rename
- [ ] GU7-T4 Quick navigation:
  - click to navigate
  - indicate broken links
- Done when: power users can quickly access favorite locations.

### Epic GU8 — Clipboard operations
- [ ] GU8-T1 Cut/Copy/Paste commands:
  - keyboard shortcuts (Ctrl+X/C/V)
  - context menu actions
- [ ] GU8-T2 Windows clipboard integration:
  - CF_HDROP format for interop
- [ ] GU8-T3 Visual feedback:
  - cut files shown as dimmed/faded
- Done when: clipboard works seamlessly with Windows Explorer.

### Epic GU9 — GUI polish (v1.5 focus)
- [ ] GU9-T1 Theme system (light/dark)
- [ ] GU9-T2 Accessibility basics (keyboard nav + focus states)
- [ ] GU9-T3 Performance profiling + bundle optimizations
- [ ] GU9-T4 Properties dialog (full, editable)
- [ ] GU9-T5 Preview pane:
  - image preview
  - text/code preview with syntax highlighting
  - toggle visibility
- [ ] GU9-T6 Tabs within panes:
  - multiple directories per pane
  - tab bar UI
  - tab management (new, close, reorder)
- Done when: GUI feels "Windows-first modern" and responsive.

### Epic GU10 — Icon system
- [ ] GU10-T1 SVG icon component:
  - load and render SVG icons
  - size variants (16px, 24px, 32px)
- [ ] GU10-T2 Icon mapping:
  - file extension → icon mapping
  - folder state icons (empty, has contents)
  - special icons (symlink, hidden, system)
- [ ] GU10-T3 Icon pack integration:
  - dev-oriented icons (code files, configs)
  - common file types (docs, images, audio, video, archives)
- Done when: all file types have appropriate icons.

---

## Milestone 5 — ZManager v1 release

### Epic R0 — Packaging + release process
- [ ] R0-T1 Portable ZIP packaging:
  - self-contained executable
  - bundled assets and config template
  - README with quick start
- [ ] R0-T2 Versioning + release notes template
- [ ] R0-T3 Crash dump integration:
  - include crash reporter in package
  - document how to submit crash reports
- [ ] R0-T4 GitHub releases automation:
  - CI builds release artifacts
  - automatic changelog generation
- Done when: v1 can be downloaded and run by users with minimal friction.

### Epic R1 — v1 acceptance & stabilization
- [ ] R1-T1 Run acceptance tests checklist
- [ ] R1-T2 Fix top crashes + data-loss risks (blockers)
- [ ] R1-T3 Performance regression gate thresholds (initial)
- Done when: release criteria are met and sign-off checklist is complete.

---

## Milestone 6 — Bug fixing + refinements (post-v1)

- [ ] P1-P0 Fix data integrity / transfer correctness bugs first
- [ ] P1-P1 Improve progress accuracy and UI smoothness
- [ ] P1-P2 Expand diagnostics: “copy job report” export (optional)

---

## Milestone 7 — ZManager v1.5 release

- [ ] V15-1 GUI considered stable (parity for core workflows)
- [ ] V15-2 Conflict UX polished
- [ ] V15-3 Performance optimizations (directory virtualization, caching improvements)
- [ ] V15-4 Windows installer (MSI or NSIS):
  - Start menu shortcuts
  - Uninstaller
  - Optional file associations
- [ ] V15-5 Preview pane (images, text, code)
- [ ] V15-6 Tabs within panes
- [ ] V15-7 Full properties dialog (editable attributes)

---

## Milestone 8 — Enhancements/refinements

Candidate epics (pick based on user feedback):
- [ ] E+: Bulk rename (rules/regex)
- [ ] E+: Hash/verify flows
- [ ] E+: Quick actions / command palette in GUI
- [ ] E+: Custom themes / theme editor
- [ ] E+: Keyboard macro recording

---

## Milestone 9 — Linux support

### Epic L0 — Portability pass
- [ ] L0-T1 Abstract OS-specific transfer backend
- [ ] L0-T2 Validate traversal/watching behavior:
  - `notify` is cross-platform, but semantics differ per OS. [web:71]
- [ ] L0-T3 Packaging approach for Linux distributions
- Done when: core + TUI run reliably on Linux and transfers have a Linux backend.

---

## Milestone 10 — ZManager v2 release

- [ ] V2-1 Linux support stable
- [ ] V2-2 Refined plugin/extension points (if chosen)
- [ ] V2-3 Long-term maintenance plan (compat matrix, CI, release cadence)

---

## Suggested ticket sizing rules
- Small: 0.5–1 day (single module change, tests included)
- Medium: 2–4 days (new module + integration)
- Large: split (if >1 week, break into smaller tickets with incremental deliverables)
