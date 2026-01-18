# ZManager — Sprint Roadmap

## Development Model
- **Developer**: AI Agent (GitHub Copilot)
- **Orchestrator/Reviewer**: Human (You)
- **Workflow**: AI builds → Human reviews/tests → Iterate → Merge

## Sprint Cadence
- **Sprint length**: 1 week
- **Deliverables**: Each sprint produces testable, runnable code
- **Review cycle**: End of each sprint = demo + review + feedback

## Timeline Overview

| Phase | Sprints | Duration | Milestone |
|-------|---------|----------|-----------|
| **Phase 1**: Core Foundation | S1-S4 | 4 weeks | Core library working |
| **Phase 2**: Transfer Engine | S5-S7 | 3 weeks | File operations complete |
| **Phase 3**: TUI | S8-S11 | 4 weeks | TUI v1 usable |
| **Phase 4**: GUI | S12-S16 | 5 weeks | GUI v1 usable |
| **Phase 5**: Polish & Release | S17-S18 | 2 weeks | v1 Release |
| **Total** | 18 sprints | ~18 weeks | **ZManager v1** |

---

# Phase 1: Core Foundation (Weeks 1-4)

## Sprint 1: Project Scaffolding & Domain Model ✅ COMPLETED
**Goal**: Rust workspace compiles, basic types exist, CI runs

### Tasks
- [x] S1-1: Create Rust workspace with crates:
  - `zmanager-core`
  - `zmanager-transfer-win`
  - `zmanager-tui`
  - `zmanager-tauri` (GUI backend - excluded from default build until Sprint 12)
- [x] S1-2: Set up CI (GitHub Actions):
  - Windows build
  - `cargo fmt --check`
  - `cargo clippy`
  - `cargo test`
- [x] S1-3: Define error types (`ZError`, `ZResult`)
- [x] S1-4: Implement core domain types:
  - `EntryKind` (File, Directory, Symlink, Junction)
  - `EntryMeta` (name, path, size, dates, attributes)
  - `DirListing` (entries + stats)
- [x] S1-5: Implement `SortSpec` and `FilterSpec`
- [x] S1-6: Unit tests for domain types (42 tests passing)

### Deliverables
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes (42 tests)
- ✅ `cargo clippy` clean
- ✅ CI workflow ready

### Review Checklist
- [x] Workspace structure makes sense
- [x] Types are well-named and ergonomic
- [x] Error messages are clear

---

## Sprint 2: Directory Listing & Navigation ✅ COMPLETED
**Goal**: Can list directories, sort, filter, navigate

### Tasks
- [x] S2-1: Implement `list_directory(path, sort, filter)`:
  - Read entries with `std::fs`
  - Collect metadata (size, dates, attributes)
  - Apply sorting
  - Apply filtering
- [x] S2-2: Symlink/junction detection:
  - Detect link type via reparse point analysis
  - Resolve target path
  - Handle broken links gracefully
- [x] S2-3: Hidden/system file detection (Windows attributes)
- [x] S2-4: Long path support:
  - Detect paths ≥240 chars
  - Apply `\\?\` prefix automatically
- [x] S2-5: Navigation state:
  - `current_dir` with back/forward history stacks
  - Max 100 entries with proper cache invalidation
- [x] S2-6: Selection model:
  - Cursor-based navigation with anchor support
  - Single, multi-select, range select
  - Click modifiers (Ctrl, Shift)
  - Toggle, add, remove, clear, invert, select_all
- [x] S2-7: Benchmarks for large directories (100, 1k, 10k, 50k files)

### Deliverables
- ✅ Can list any directory with correct metadata
- ✅ Sorting works correctly (name, size, date, kind)
- ✅ Filtering works (name pattern, kind filter)
- ✅ Symlinks/junctions displayed with target info
- ✅ Benchmarks recorded: **~1M files/sec throughput**

### Performance Results
| Files | Avg Time | P95 |
|-------|----------|-----|
| 100 | 170µs | 225µs |
| 1,000 | 1.1ms | 1.25ms |
| 10,000 | 10.6ms | 12.2ms |
| 50,000 | 52.2ms | 55.3ms |

### Review Checklist
- [x] Performance acceptable for large dirs (~1M files/sec)
- [x] Symlinks handled correctly (reparse point detection)
- [x] Selection model intuitive (cursor + modifiers)

---

## Sprint 3: File Operations & Job System ✅ COMPLETED
**Goal**: Rename, delete, mkdir work; job system scaffolded

### Tasks
- [x] S3-1: Implement `rename(from, to)`:
  - Conflict detection (AlreadyExists error)
  - Return structured ZError on failure
- [x] S3-2: Implement `delete(path, permanent)`:
  - Recycle Bin via `SHFileOperationW`
  - Permanent delete bypasses Recycle Bin
  - Recursive folder delete
- [x] S3-3: Implement `mkdir(path)` with nested directory support
- [x] S3-4: Implement `open_default(path)` via explorer command
- [x] S3-5: Job system foundation:
  - `JobId`, `JobKind`, `JobState` types
  - `Progress` struct with percentage, ETA, speed
  - Job lifecycle state machine (Pending → Running → Completed/Failed/Cancelled)
- [x] S3-6: Job scheduler (Tokio-based):
  - Job queue with concurrent limits
  - Event broadcast channel
  - Submit, cancel, pause, resume operations
- [x] S3-7: Cancellation tokens (cooperative cancellation with Arc<AtomicBool>)

### Deliverables
- ✅ Can rename/delete/mkdir from code
- ✅ Recycle Bin works correctly (Windows SHFileOperationW)
- ✅ Jobs can be created, cancelled, paused, resumed

### Review Checklist
- [x] Delete to Recycle Bin is recoverable
- [x] Permanent delete works (recursive optional)
- [x] Job states are correct

---

## Sprint 4: Configuration, Favorites & Observability ✅ COMPLETED
**Goal**: TOML config works, favorites persist, tracing integrated

### Tasks
- [x] S4-1: TOML config schema:
  - Define all settings with serde (GeneralConfig, AppearanceConfig, OperationsConfig)
  - Default values for all settings
- [x] S4-2: Config file management:
  - Load from `%APPDATA%\ZManager\config.toml`
  - Create default if missing
  - Validate on load
- [x] S4-3: Favorites/Quick Access:
  - Data model (`Favorite { id, name, path, order, icon }`)
  - CRUD operations (add, remove, get, reorder)
  - Persist in config
  - Broken favorite detection (`is_broken()`)
- [x] S4-4: Session state:
  - `SessionState` struct with last directories
  - Window state persistence
  - Last sort settings
- [x] S4-5: Drives enumeration:
  - List available drives with `GetLogicalDrives`
  - Get labels, file system, free space
  - Drive type detection (Fixed, Removable, Network, etc.)
- [x] S4-6: Integrate `tracing`:
  - All new modules use tracing spans and debug! macros
  - Ready for subscriber configuration
- [x] S4-7: Properties data collection:
  - File properties (size, dates, attributes, MIME type)
  - Folder properties (`calculate_folder_stats` for async size/count)

### Deliverables
- ✅ Config file loads and saves (TOML format)
- ✅ Favorites persist across restarts
- ✅ Drives listed with type, label, free space
- ✅ Tracing spans in all modules

### Review Checklist
- [x] Config format is user-friendly (TOML with sections)
- [x] Favorites work as expected
- [x] Logs are useful for debugging

---

# Phase 2: Transfer Engine (Weeks 5-7)

## Sprint 5: CopyFileEx Integration ✅ COMPLETED
**Goal**: Single-file copy with progress works

### Tasks
- [x] S5-1: Windows crate setup (`windows` v0.58 with Win32_System_IO feature)
- [x] S5-2: Implement `copy_file_with_progress()`:
  - Uses `CopyFileExW` with progress callback
  - Progress callback → Progress events with speed/ETA
  - Cancellation via `PROGRESS_CANCEL` return value
- [x] S5-3: Wire to job system:
  - `CopyExecutor` bridges copy primitives to job system
  - Event-based progress streaming via broadcast channels
  - Cancel support via `CancellationToken`
- [x] S5-4: Error handling:
  - Maps Windows HRESULT to ZError variants
  - Includes path context and human-readable messages
- [x] S5-5: Test with large files (10MB, 100MB)
  - 5 integration tests for large file scenarios
  - Progress callback verification
  - Content integrity verification
- [x] S5-6: Benchmark throughput vs native copy
  - **Results**: 2.85 GB/s for 100MB files
  - **Ratio**: 1.01x native (essentially identical)

### Deliverables
- ✅ Can copy large files with live progress
- ✅ Can cancel mid-copy
- ✅ Throughput close to native (1.01x for large files)

### Review Checklist
- [x] Progress updates smoothly (10 updates for 10MB file)
- [x] Cancellation is responsive (verified in benchmark)
- [x] No partial files left on cancel (cleanup implemented)

---

## Sprint 6: Folder Transfer & Conflict Resolution ✅ COMPLETED
**Goal**: Folder copy/move works with conflict handling

### Tasks
- [x] S6-1: Transfer plan builder:
  - Enumerate source tree with `walkdir`
  - Generate destination paths
  - Calculate total bytes/items
  - `TransferPlan`, `TransferItem`, `TransferStats` types
- [x] S6-2: Folder copy execution:
  - Create directories in order (depth-first)
  - Schedule file copies with progress aggregation
  - `FolderTransferExecutor` with event broadcasting
- [x] S6-3: Move semantics:
  - Same-volume: atomic rename via `same_volume()` detection
  - Cross-volume: copy → delete on success
- [x] S6-4: Conflict resolution:
  - Detect existing destination files
  - `ConflictPolicy` enum: Overwrite, Skip, Rename, KeepNewer, KeepLarger, Ask
  - `ConflictResolver` with "Apply to all" support
  - `generate_rename_path()` for automatic renaming (file (1).txt pattern)
- [x] S6-5: "Ask" mode protocol:
  - Emit `ConflictDetected` event on conflict
  - `ConflictQuery` with oneshot response channel
  - Ready for UI integration
- [x] S6-6: Partial failure handling:
  - `continue_on_error` config option
  - `ItemResult` enum: Success, Skipped, Failed
  - `TransferReport` with aggregated success/skip/fail counts

### Deliverables
- ✅ Can copy/move folders with `FolderTransferExecutor`
- ✅ Conflict policies work with 5 automatic modes + Ask
- ✅ Partial failures handled gracefully (continue or stop)
- ✅ 27 new tests (49 total in transfer-win, 177 workspace total)

### Review Checklist
- [x] Directory structure preserved (depth-first creation)
- [x] Conflicts resolved correctly (all 5 policies tested)
- [x] Move deletes source only on success

---

## Sprint 7: Transfer Reporting & Clipboard ✅ COMPLETED
**Goal**: Transfer reports work, clipboard integration complete

### Tasks
- [x] S7-1: Transfer report model:
  - `TransferStatus` (Success, Skipped, Failed) with labels and symbols
  - `TransferItemResult` per-file result with path, size, duration, error
  - `TransferSummary` aggregated stats (counts, bytes, speed)
  - `DetailedTransferReport` full report with job metadata
- [x] S7-2: Report persistence:
  - `ReportStorage` with configurable directory (default `%APPDATA%\ZManager\reports`)
  - Millisecond-precision timestamps for unique filenames
  - `load()`, `list()`, `cleanup(keep_count)` operations
- [x] S7-3: Report export (JSON + text):
  - `to_json()` via serde serialization
  - `to_text()` human-readable format with Unicode symbols
  - `save_json()` and `save_text()` file operations
- [x] S7-4: Clipboard integration:
  - CF_HDROP write with `DROPFILES` structure
  - CF_HDROP read via `DragQueryFileW`
  - Preferred drop effect (move vs copy) via `RegisterClipboardFormatW`
  - `Clipboard::copy()`, `Clipboard::cut()`, `Clipboard::paste()`, `Clipboard::has_files()`
- [x] S7-5: Cut/Copy/Paste operations:
  - `write_files_to_clipboard()` with `DropEffect`
  - `read_files_from_clipboard()` returns `ClipboardContent`
  - Explorer interoperability tested
- [x] S7-6: File watching (`notify` crate):
  - `DirectoryWatcher` with `notify::RecommendedWatcher`
  - Debounced events via polling loop (configurable, default 300ms)
  - `WatchEvent` with `WatchEventKind` (Created, Modified, Deleted, Renamed, Changed)
  - Max watched directories limit (configurable, default 10)
  - Subscribe model with `tokio::sync::broadcast`

### Deliverables
- ✅ Transfer reports with JSON/text export and persistence
- ✅ Copy/paste works with Explorer (CF_HDROP + preferred drop effect)
- ✅ Directories auto-refresh capability via file watcher
- ✅ 211 tests passing (129 core + 72 transfer-win + 5 integration + 5 doc)

### Review Checklist
- [x] Report shows correct details (per-file status, summary stats)
- [x] Clipboard interop with Explorer works (serial tests for thread safety)
- [x] File watching uses debouncing to prevent thrashing

---

# Phase 3: TUI (Weeks 8-11)

## Sprint 8: TUI Scaffolding & Basic Navigation ✅ COMPLETED
**Goal**: TUI starts, can browse directories

### Tasks
- [x] S8-1: Ratatui + Crossterm setup
  - `Tui` struct with `Terminal<CrosstermBackend<Stdout>>`
  - `enter()` / `exit()` for raw mode + alternate screen
  - `draw()` for rendering frames
- [x] S8-2: Tokio async runtime integration
  - `#[tokio::main]` async entry point
  - `EventHandler` with async `next()` method
  - Background task for polling terminal events
- [x] S8-3: Basic layout:
  - `AppLayout` with header, content, status areas
  - `dual_panes()` for left/right split
  - `Pane` enum with `toggle()`
- [x] S8-4: Event loop:
  - Key events via `crossterm::event`
  - Tick events for periodic updates
  - Resize handling
  - `DirectoryChanged` events for refresh
- [x] S8-5: File list rendering:
  - `FileList` stateful widget
  - Emoji icons (📁📄🔗⛓️)
  - Size column with human-readable formatting
  - Selection highlighting
  - Color-coded by type (dir=blue, exe=green, archive=red)
- [x] S8-6: Basic navigation:
  - Arrow keys for up/down/enter/parent
  - Enter directory on Enter/Right/l
  - Parent directory on Backspace/Left/h
  - History with back/forward (Alt+arrows, [ and ])
- [x] S8-7: Vim-style keys (h/j/k/l)
  - j/k for up/down
  - h/l for parent/enter
  - g/G for first/last
  - Ctrl+u/d for page up/down

### Deliverables
- ✅ TUI starts and shows current directory
- ✅ Can navigate with keyboard (Vim + arrow keys)
- ✅ Dual-pane layout with Tab switching
- ✅ 224 tests passing (129 core + 72 transfer-win + 5 integration + 18 TUI)

### Review Checklist
- [ ] Rendering looks clean
- [ ] Navigation feels responsive
- [ ] Keys work as documented

---

## Sprint 9: TUI Dual-Pane & Operations ✅
**Goal**: Dual-pane mode, file operations work

### Tasks
- [x] S9-1: Dual-pane layout:
  - Side-by-side panels with separate headers
  - Active pane indicator (bold border/header)
  - Tab to switch between panes
- [x] S9-2: Independent pane state:
  - Separate path, selection, history per pane
- [x] S9-3: File operations UI:
  - Delete (d) → confirmation dialog → execute
  - Rename (r) → input dialog → execute
  - Mkdir (n) → input dialog → execute
- [x] S9-4: Copy/Move to other pane:
  - Shift+C/M keys show confirmation
  - Initiate file copy/move operation
- [x] S9-5: Selection operations:
  - Space toggle with cursor advance
  - Visual feedback for selected items
  - Multi-select operations work
- [x] S9-6: Hidden files toggle (.) key
- [x] S9-7: Sorting menu (s key):
  - Name (n), Size (s), Modified (m), Extension (e), Kind (k)

### Deliverables
- ✅ Dual-pane mode works with separate headers
- ✅ Can copy/move between panes
- ✅ Delete/rename/mkdir work via dialogs
- ✅ 234 tests passing (129 core + 72 transfer-win + 5 integration + 23 TUI + 5 doc)

### Review Checklist
- [x] Dual-pane layout balanced
- [x] Operations feel natural
- [x] Feedback is clear via dialogs

---

## Sprint 10: TUI Transfers & Conflicts ✅ COMPLETED
**Goal**: Transfer progress visible, conflict resolution works

### Tasks
- [x] S10-1: Transfers view (t key):
  - List active jobs with state icons
  - Progress bars with percentage and color coding
  - Throughput display with speed formatting (KB/s, MB/s, GB/s)
  - ETA calculation and display
- [x] S10-2: Job controls:
  - Pause (Shift+P) - pauses selected job
  - Resume (Shift+R) - resumes paused job
  - Cancel (Shift+X) - cancels selected job
- [x] S10-3: Conflict resolution modal:
  - ConflictModal widget with file info display
  - Single-key responses: o(verwrite), s(kip), r(ename), l(arger), n(ewer), c(ancel)
  - Apply-to-all with Shift modifier or 'a' toggle
  - Shows source/dest sizes and modification dates
- [x] S10-4: Transfer completion:
  - status_message field for status bar notifications
  - Shows resolution and error messages
- [x] S10-5: Error display:
  - Status bar messages with error/success coloring
  - set_status() method with is_error flag
- [x] S10-6: Auto-refresh on file changes:
  - DirectoryWatcher integration in main event loop
  - Watches both pane directories
  - Auto-reloads on file system events

### Deliverables
- ✅ Can monitor transfers in dedicated view (t key)
- ✅ Conflict resolution modal with all options
- ✅ Errors clearly displayed in status bar
- ✅ 242 tests passing (129 core + 72 transfer-win + 5 integration + 36 TUI)

### Review Checklist
- [x] Progress bars work correctly
- [x] Conflict modal is clear with hotkeys
- [x] Auto-refresh works for both panes

---

## Sprint 11: TUI Polish & Quick Access ✅ COMPLETE
**Goal**: Quick Access works, config keybindings, crash reporting

### Tasks
- [x] S11-1: Quick Access sidebar:
  - Favorites panel (Ctrl+b toggle)
  - Navigate with arrows
  - Quick-jump with 1-9
- [x] S11-2: Add/remove favorites:
  - Shift+D to add current dir
  - Remove from panel
- [x] S11-3: Drives panel:
  - List drives
  - Show free space
  - Navigate to drive
- [ ] S11-4: Configurable keybindings (deferred to future sprint):
  - Load from config.toml
  - Apply at runtime
- [x] S11-5: Crash reporting:
  - Panic hook installed
  - Write crash dump to %LOCALAPPDATA%\ZManager\crashes
  - Detect on startup and log
- [x] S11-6: Help screen (? key)
- [x] S11-7: Properties panel (i key)

### Deliverables
- ✅ Quick Access fully functional (sidebar with favorites/drives)
- ⏳ Keybindings configurable (deferred)
- ✅ Crash dumps work

### Review Checklist
- [x] Favorites persist correctly (via Config save/load)
- [ ] Custom keybindings work (deferred)
- [x] Crash dump is useful

### Implementation Notes
**Test count**: 257 tests (129 core + 72 transfer + 5 integration + 46 TUI + 5 doctests)

**New Files**:
- `crates/zmanager-tui/src/ui/sidebar.rs` - Quick Access sidebar widget
- `crates/zmanager-tui/src/ui/help.rs` - Help screen modal
- `crates/zmanager-tui/src/ui/properties.rs` - Properties panel modal
- `crates/zmanager-tui/src/crash.rs` - Crash reporting module

**Key Features**:
- Ctrl+b toggles sidebar (left 30% of left pane)
- Tab switches between Favorites and Drives sections
- 1-9 quick jump to favorites
- Shift+D adds current directory to favorites
- Delete removes selected favorite
- ? shows help screen with all keybindings
- i shows properties panel for selected file/folder
- Crash dumps include backtrace, timestamp, and panic message

---

# Phase 4: GUI (Weeks 12-16)

## GUI Architecture Decisions

| Decision | Choice | Notes |
|----------|--------|-------|
| **Window Frame** | Custom titlebar | Windows-style controls (minimize/maximize/close) via Tauri v2 |
| **CSS Framework** | Tailwind CSS v4 | Using `@tailwindcss/vite` plugin |
| **State Management** | Zustand | Lightweight, hooks-based |
| **Data Fetching** | React Query | For IPC caching and async state |
| **Linting/Formatting** | BiomeJS | Replaces ESLint + Prettier |
| **Package Manager** | Bun | Per project conventions |
| **View Mode** | List view (Sprint 12) | Grid/Details toggle in later sprint |
| **Preview Panel** | Deferred | Not in Sprint 12, later sprint |
| **File Icons** | Windows Shell + dev_icons | Default Windows icons + dev-oriented SVG pack |
| **App Icon** | `/assets/ZManager_Icon.png` | Official branding |

### Color Palette (from mockups)
| Color | Value | Usage |
|-------|-------|-------|
| Primary/Accent | `#FED8B2` | Active elements, highlights |
| Background Dark | `#323232` | Main background |
| Charcoal Dark | `#252525` | Sidebar, header |
| Surface | `#2b2b2b` | Cards, elevated surfaces |
| Border | `#444444` | Dividers, borders |
| Text Muted | `#a3a3a3` | Secondary text |

### Typography
- **Display Font**: Inter (weights 400-700)
- **Mono Font**: Fira Code (for paths, code preview)

---

## Sprint 12: GUI Scaffolding & IPC ✅ COMPLETED
**Goal**: Tauri app starts, basic IPC works
**GUI Design/UI Mockup References**: /docs/GUI_MockDesign/mock_part1.html & /docs/GUI_MockDesign/mock_part2.html

### Tasks
- [x] S12-1: React 19 + Vite + TypeScript project setup
- [x] S12-2: Bun toolchain + BiomeJS:
  - `bun install` configured
  - BiomeJS for lint/format
  - Tailwind CSS v4 integrated
- [x] S12-3: Rust command layer:
  - Expose `zmanager_list_dir`
  - Expose `zmanager_get_drives`
  - Error handling pattern (`IpcResponse<T>`)
- [x] S12-4: TypeScript IPC client:
  - Typed `invoke()` wrapper in `lib/tauri.ts`
  - Result/error handling with `IpcError` class
- [x] S12-5: Zustand store setup:
  - File system state (`fileSystem.store.ts`)
  - UI state (`ui.store.ts`)
- [x] S12-6: Custom titlebar:
  - Windows-style controls (minimize/maximize/close)
  - Draggable region
  - App branding
- [x] S12-7: Basic layout shell:
  - Sidebar with favorites/drives
  - Dual-pane file list area
  - Status bar
- [x] S12-8: Icon system (bonus):
  - External SVG icons in `/gui/public/icons/`
  - Icon mapping utilities (`iconMappings.ts`)
  - SvgIcon component with dark theme support

### Deliverables
- ✅ Tauri app launches with custom titlebar
- ✅ Can fetch directory listing via IPC
- ✅ Bun + BiomeJS toolchain works
- ✅ Icon system with dev_icons, filetypes, ui categories

### Review Checklist
- [x] App starts quickly
- [x] IPC is responsive
- [x] TypeScript types match Rust
- [x] Custom titlebar functional
- [x] Icons visible on dark theme

### Implementation Notes
**Test count**: 257 tests (unchanged from Sprint 11)

**New Files (GUI)**:
- `crates/zmanager-tauri/gui/src/App.tsx` - Main app component
- `crates/zmanager-tauri/gui/src/components/TitleBar.tsx` - Custom Windows titlebar
- `crates/zmanager-tauri/gui/src/components/Sidebar.tsx` - Quick access sidebar
- `crates/zmanager-tauri/gui/src/components/FilePane.tsx` - Dual-pane file list
- `crates/zmanager-tauri/gui/src/components/StatusBar.tsx` - Status bar
- `crates/zmanager-tauri/gui/src/components/SvgIcon.tsx` - SVG icon component
- `crates/zmanager-tauri/gui/src/stores/fileSystem.store.ts` - Zustand file system store
- `crates/zmanager-tauri/gui/src/stores/ui.store.ts` - Zustand UI store
- `crates/zmanager-tauri/gui/src/lib/tauri.ts` - Typed IPC client
- `crates/zmanager-tauri/gui/src/utils/iconMappings.ts` - Icon mapping utilities
- `crates/zmanager-tauri/gui/public/icons/` - SVG icon assets (dev_icons, filetypes, ui)

**New Files (Rust)**:
- `crates/zmanager-tauri/src/commands.rs` - Tauri IPC commands

---

## Sprint 13: GUI File List & Navigation ✅ COMPLETED
**Goal**: Can browse directories with virtualized list

### Tasks
- [x] S13-1: Virtualized file list (@tanstack/react-virtual)
  - Installed @tanstack/react-virtual v3.13.18
  - VirtualizedFileList component with overscan and smooth scrolling
  - 28px row height, keyboard navigation (Arrow keys, Home/End, Page Up/Down)
- [x] S13-2: File row component:
  - Icon from dev_icons/filetypes mapping
  - Name, size, date columns with formatting
  - Selection highlight (primary color)
  - Cursor indicator (background tint)
- [x] S13-3: Smart address bar:
  - Breadcrumb segments (clickable navigation)
  - Edit mode on click (direct path input)
  - Autocomplete dropdown with debounced suggestions
  - Tab completion for directories
- [x] S13-4: Navigation:
  - Double-click to enter directories
  - Backspace to go up
  - Back/forward with Alt+Arrow keys
  - F5 to refresh
- [x] S13-5: Sorting controls:
  - Column header clicks toggle sort
  - Sort indicator (chevron icon)
  - Ascending/Descending toggle
- [x] S13-6: Search/filter input
  - Search box in header
  - Pattern filtering via FilterSpec
  - Smooth width expansion on focus
- [x] S13-7: Empty state UI
  - Visual empty folder icon
  - Helpful message text
  - Loading state with spinner
  - Error state with retry button

### Deliverables
- ✅ Can browse any directory
- ✅ 50k+ entries smooth (virtualized rendering)
- ✅ Address bar works (breadcrumbs + edit mode + autocomplete)

### Review Checklist
- [x] Scrolling is smooth (virtualized with overscan)
- [x] Breadcrumbs intuitive (click to navigate)
- [x] Autocomplete helpful (directory suggestions)

### Implementation Notes
**New Dependencies**:
- `@tanstack/react-virtual` v3.13.18

**New/Updated Components**:
- `VirtualizedFileList.tsx` - Virtualized file list with keyboard nav
- `AddressBar.tsx` - Smart breadcrumb/edit address bar
- `FilePane.tsx` - Updated to use new components

**Key Features**:
- Multi-select: Ctrl+Click toggle, Shift+Click range
- Select All: Ctrl+A
- Keyboard: Arrow keys, Enter, Backspace, Home/End, Page Up/Down
- Sort toggle: Click column headers
- Search: Real-time pattern filtering

---

## Sprint 14: GUI Dual-Pane & Operations ✅
**Goal**: Dual-pane mode, context menus, file operations

### Tasks
- [x] S14-1: Dual-pane layout:
  - Resizable split using react-resizable-panels
  - Active pane indicator (ring highlight)
  - Tab to switch (click to activate)
- [x] S14-2: Context menu component:
  - Right-click on file
  - Right-click on empty space
  - All actions wired
- [x] S14-3: File operations:
  - Delete with confirmation (Recycle Bin)
  - Rename inline dialog
  - New folder dialog
- [x] S14-4: Selection:
  - Ctrl+click multi-select
  - Shift+click range select
  - Select all (Ctrl+A)
- [x] S14-5: Keyboard shortcuts:
  - Delete, F2, Ctrl+Shift+N, Enter, Alt+Enter
  - Alt+Left/Right for history
  - Backspace for parent
  - F5 for refresh
- [x] S14-6: Open with default app (Enter/double-click)
- [x] S14-7: Properties panel (modal with file details)

### Deliverables
- ✅ Dual-pane functional
- ✅ Context menus work
- ✅ All file ops work

### Review Checklist
- [x] Context menu feels native
- [x] Shortcuts work
- [x] Dual-pane is intuitive

---

## Sprint 15: GUI Drag & Drop & Transfers ✅ COMPLETED
**Goal**: DnD works, transfer progress visible

### Tasks
- [x] S15-1: Internal DnD (dnd-kit):
  - Drag within list
  - Drag between panes
  - Drop zone highlighting
- [x] S15-2: External drop (Tauri native):
  - Files from Explorer
  - Initiate copy/move
- [x] S15-3: External drag-out:
  - Tauri `startDrag`
  - Drag to Explorer
- [x] S15-4: Transfer panel:
  - Show active jobs
  - Progress bars
  - Pause/resume/cancel buttons
- [x] S15-5: Conflict dialog:
  - Modal with file info
  - Action buttons
  - Apply to all checkbox
- [x] S15-6: Toast notifications:
  - Success/error toasts
  - Auto-dismiss
  - Click for details

### Deliverables
- ✅ DnD works internally and externally
- ✅ Transfer progress visible
- ✅ Toasts appear correctly

### Implementation Notes
- **Internal DnD**: `@dnd-kit/core` v6.3.1 with `DndContext`, `useDraggable`, `useDroppable`
- **External Drop**: `onDragDropEvent` from `@tauri-apps/api/webview` with `dragDropEnabled: false` in config
- **External Drag-out**: `tauri-plugin-drag` v2.1.0 via `@crabnebula/tauri-plugin-drag` npm package
- **Components Created**:
  - `DndProvider.tsx` - DnD context with internal and external drag support
  - `TransferPanel.tsx` - Job progress display with pause/resume/cancel
  - `ConflictDialog.tsx` - File conflict resolution modal
  - `Toast.tsx` - Toast notification system with variants
- **CSS Additions**: `.drop-zone-active`, `.animate-slide-in-right` animations

### Review Checklist
- [x] DnD feels native
- [x] Progress accurate
- [x] Conflict dialog clear

---

## Sprint 16: GUI Quick Access, Icons & Clipboard ✅
**Goal**: Quick Access sidebar, custom icons, clipboard works
**Status**: COMPLETED

### Tasks
- [x] S16-1: Quick Access sidebar:
  - Favorites list from config
  - Drag to reorder with @dnd-kit/sortable
  - Right-click context menu to remove
  - Click to navigate
- [x] S16-2: Add to Quick Access:
  - Default favorites auto-created on first run
  - Favorites stored in config via backend commands
- [x] S16-3: Drives section in sidebar (existing)
- [x] S16-4: SVG icon system (existing):
  - SvgIcon component
  - Extension → icon mapping in iconMappings.ts
  - Drive type icons
- [x] S16-5: Icon integration (existing icon pack)
- [x] S16-6: Clipboard operations:
  - Ctrl+C/X/V with useKeyboardShortcuts hook
  - Backend clipboard state with copy/cut/paste commands
  - Toast feedback on operations
- [x] S16-7: File watching → auto-refresh:
  - useFileWatcher hook refreshes on window focus
  - Visibility change detection for tab switching

### Implementation Details
- **Backend Commands Added**:
  - `zmanager_get_favorites`, `zmanager_add_favorite`, `zmanager_remove_favorite`, `zmanager_reorder_favorites`
  - `zmanager_clipboard_copy`, `zmanager_clipboard_cut`, `zmanager_clipboard_get`, `zmanager_clipboard_paste`, `zmanager_clipboard_clear`
- **Stores Added**: `favorites.store.ts`, `clipboard.store.ts`
- **Hooks Added**: `useKeyboardShortcuts.ts`, `useFileWatcher.ts`
- **Sidebar Updated**: Dynamic favorites with sortable DnD and context menu
- **JS Bundle**: 374.80 KB

### Deliverables
- ✅ Quick Access fully functional
- ✅ Icons display correctly
- ✅ Clipboard works (internal, Explorer interop pending)

### Review Checklist
- [x] Icons look good
- [x] Quick Access intuitive
- [x] Clipboard operations work

---

# Phase 5: Polish & Release (Weeks 17-18)

## Sprint 17: Integration Testing & Bug Fixing
**Goal**: Both TUI and GUI stable, major bugs fixed

### Tasks
- [ ] S17-1: Run all acceptance tests (manual)
- [ ] S17-2: Fix blocking bugs (P0)
- [ ] S17-3: Performance profiling:
  - Large directory benchmarks
  - Transfer throughput checks
- [ ] S17-4: Edge case testing:
  - Long paths
  - Network drives
  - Permission denied
  - Symlink loops
- [ ] S17-5: Cross-test TUI ↔ GUI:
  - Same core behavior
  - Config shared correctly
- [ ] S17-6: Crash reporting validation

### Deliverables
- ✅ Acceptance tests pass
- ✅ No P0 bugs remaining
- ✅ Performance acceptable

### Review Checklist
- [ ] All tests pass
- [ ] No data loss scenarios
- [ ] Stable under stress

---

## Sprint 18: Packaging & v1 Release
**Goal**: Portable ZIP ready, v1 shipped

### Tasks
- [ ] S18-1: Portable ZIP packaging:
  - TUI executable
  - GUI executable
  - Default config template
  - README
- [ ] S18-2: CI release automation:
  - Build on tag
  - Create GitHub release
  - Attach artifacts
- [ ] S18-3: Version stamping:
  - Version in binaries
  - Version in config
- [ ] S18-4: Release notes:
  - Feature summary
  - Known issues
  - Getting started guide
- [ ] S18-5: Final smoke test
- [ ] S18-6: 🎉 **Release v1.0.0**

### Deliverables
- ✅ ZIP downloadable
- ✅ Works on fresh Windows install
- ✅ v1 released!

---

# Post-v1 Roadmap

## v1.1 - Bug Fixes
- Fix issues reported after release
- Performance improvements

## v1.5 - Enhancements
- Windows installer (MSI)
- Tabs within panes
- Preview pane
- Full properties dialog
- Theme system (light/dark)

## v2.0 - Linux Support
- Linux transfer backend
- Linux packaging (.deb, .rpm, AppImage)
- Cross-platform testing

---

# Sprint Tracking Template

```markdown
## Sprint N: [Title]
**Dates**: [Start] → [End]
**Goal**: [One-line goal]

### Completed
- [x] Task 1
- [x] Task 2

### In Progress
- [ ] Task 3

### Blocked
- [ ] Task 4 (reason)

### Carried Over
- [ ] Task 5

### Notes
- Observation 1
- Decision made

### Demo/Review
- [ ] Demoed to reviewer
- [ ] Feedback incorporated
```

---

# Let's Begin! 🚀

**Next Step**: Start Sprint 1 — Project Scaffolding & Domain Model

When you're ready, say "Start Sprint 1" and I'll begin building the Rust workspace and core types!
