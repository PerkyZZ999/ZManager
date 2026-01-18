<!-- File: PRD_ZManager.md -->

# ZManager — PRD (Windows v1)

## Summary
ZManager is a high-performance file manager built in Rust (Edition 2024) with a shared core and two frontends: a TUI (Ratatui) first, then a GUI (Tauri v2 + React 19).  
v1 targets Windows only; Linux support is planned later; macOS is explicitly not supported.

## Goals
- **Stability-first**: Rock-solid reliability — no crashes, no data loss, predictable behavior.
- **Performance-first**: Large folders stay responsive; transfers approach OS-level throughput.
- Fast, reliable file operations with an integrated Windows-native transfer engine.
- Consistent behavior across TUI and GUI via a shared Rust Core.
- Developer-friendly architecture: testable, observable, benchmarked.
- Offer both TUI and GUI — a unique value proposition no other file manager provides.

## Non-goals (v1)
- macOS support (never).
- Cloud sync, accounts, collaboration.
- Full explorer shell replacement (file associations, shell extensions) unless later required.
- Built-in archive management (zip/7z) unless prioritized later.
- Plugin/extension system (deferred to v2+).

## Target users
- Power users who prefer keyboard-driven navigation and batch operations.
- Developers/sysadmins who want predictable performance, strong diagnostics, and automation-friendly workflows.

## Product principles
- “No UI stalls”: long work happens in jobs, never blocking rendering.
- “Core owns truth”: both UIs call the same Core commands and consume the same event stream.
- “Observable by default”: structured logs and spans around every operation path.- "Stability over features": correctness and reliability always trump new functionality.
- "Windows-native feel": respect platform conventions (Recycle Bin, shortcuts, context menus).
## Platforms
- v1: Windows 10/11 (primary: Windows 11).
- Later: Linux (v2); no macOS.

## Key components
### ZManager Core (Rust)
- Domain model: entries, metadata, selection model, navigation state.
- Operations: create, rename, delete (Recycle Bin + permanent), copy, move, enumerate, search/filter/sort.
- Job system: cancellable, progress-producing tasks.
- File watching: live directory updates via `notify` crate (wraps ReadDirectoryChangesW on Windows).
- Diagnostics: tracing + metrics hooks.
- Symlink/junction handling: detect, display, and operate on links correctly.

### ZManager Transfer Engine (Windows-native)
- File copy/move based on Win32 CopyFileEx to leverage progress callbacks and cancellation. [web:28]
- Folder operations implemented as a job graph: enumerate → schedule file jobs → aggregate progress.
- Move semantics: same-volume = atomic rename; cross-volume = copy + delete on success.
- Pause behavior: stops scheduling new file tasks; in-flight files complete or cancel.
- Transfer report: structured per-file result list (success/failed/skipped) for user review.

### ZManager TUI
- Ratatui UI layer + Crossterm backend. [web:3][web:12]
- Tokio-driven event loop for responsive UI while jobs run. [web:54]
- Dual-pane mode as a core feature (not optional).
- Conflict resolution via bottom-bar modal prompt (single-keypress responses).
- Quick Access sidebar with user-defined favorites/bookmarks.
- Status bar: left=current path, center=selection count + size, right=free space + messages.
- TOML-based configuration for keybindings and preferences.
- Tabs within panes (v1.5+): multiple directories per pane.

### ZManager GUI
- Tauri v2 application shell with Rust backend and web frontend. [web:52]
- React 19 frontend with @tanstack/react-virtual for large directory virtualization.
- Bun is mandatory for frontend package management, runtime and test runner. [web:109][web:107]
- Drag & drop: dnd-kit for internal DnD + Tauri native APIs for external file drops.
- Context menus: right-click actions for common file operations.
- Dual-pane mode as a core feature.
- Quick Access sidebar with user-defined favorites/bookmarks.
- Smart address bar: clickable breadcrumbs + editable mode with autocomplete.
- Toast notifications for errors and operation feedback.
- Windows clipboard integration for copy/paste between ZManager and Explorer.
- Custom SVG icon pack (dev-oriented + common file types).
- Preview pane for files (v1.5+): images, text, code with syntax highlighting.

## Success metrics (initial)
- Responsiveness: UI stays interactive during large directory scans and long transfers.
- Reliability: operations produce actionable errors and never silently lose data.
- Performance: transfer throughput close to OS-level copy for typical local disk scenarios.
- Adoption signals: users choose ZManager for daily workflows (TUI) and later GUI.

## Milestones / roadmap
- ZManager Core
- ZManager File Transfer (Native & Fast)
- ZManager TUI
- ZManager GUI
- ZManager v1 release
- Bug fixing + refinements
- ZManager v1.5 release
- Enhancements + refinements
- Linux support
- ZManager v2 release

## Key risks
- Windows edge cases in file ops: permissions, long paths (>260 chars), locked files, special attributes.
- Transfer correctness vs speed tradeoffs (must prioritize correctness).
- Maintaining strict parity between TUI and GUI behaviors.
- Keeping Bun-only toolchain friction low for contributors (CI and scripts must be explicit).
- File watching reliability: debouncing, handling rapid changes, avoiding UI thrashing.
- Symlink/junction edge cases: broken links, circular references, permission issues.
- Clipboard format compatibility with Windows Explorer.
- Address bar autocomplete performance with deep directory structures.

## Crash Reporting
- Local crash dump on panic (written to user data directory).
- On next startup, detect crash and offer to view/export crash report.
- Optional: integrate lightweight crash reporting service for aggregated insights.
- No telemetry or usage analytics — strictly crash data only, user-initiated.

## Dependencies / constraints
- Windows-only v1 transfer optimization can use Win32 via Rust windows crate bindings. [web:66]
- GUI IPC must use Tauri commands (`invoke`) and events (`listen`) patterns. [web:119][web:114]
