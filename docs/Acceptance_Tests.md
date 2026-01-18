<!-- File: Acceptance_Tests.md -->

# ZManager — Acceptance Tests (v1)

## A) Core browsing
AT-A1: Large directory responsiveness
- Given a folder with 50k+ entries
- When listing and sorting by name
- Then UI remains responsive (no multi-second freeze) and results render incrementally

AT-A2: Sorting correctness
- Given mixed files and folders
- When sorting by size/mtime/type
- Then ordering matches expected rules consistently

AT-A3: Filter/search
- Given a directory listing
- When a filter string is applied
- Then only matching entries appear and selection remains predictable

## B) File operations
AT-B1: Rename
- When renaming to an existing name
- Then conflict is reported and no data is lost

AT-B2: Delete to Recycle Bin
- When deleting a file/folder without Shift held
- Then item is moved to Recycle Bin and can be restored

AT-B3: Permanent delete
- When deleting with Shift modifier held
- Then item is permanently deleted (bypasses Recycle Bin)

AT-B4: Delete recursive
- When deleting a folder recursively
- Then the folder is removed and errors include the path if a file is locked

AT-B5: Symlink handling
- Given a directory containing symlinks and junctions
- When displaying the listing
- Then symlinks are visually indicated and show target path
- When deleting a symlink
- Then only the link is deleted, not the target

## C) Transfer engine (Windows)
AT-C1: CopyFileEx progress streaming
- Given a 10GB file copy job
- When starting a copy
- Then progress updates arrive continuously and can be canceled mid-transfer (CopyFileEx supports progress routine + cancellation semantics). [web:28]

AT-C2: Multi-file folder copy
- Given a folder tree with many files
- When copying it
- Then overall progress reflects aggregated bytes/items and UI remains responsive

AT-C3: Conflict policy
- Given destination already contains some files
- When conflict mode is overwrite/skip/rename
- Then behavior matches the selected policy across the full job

AT-C4: Transfer report
- Given a transfer job with mixed results (some success, some failures, some skips)
- When job completes
- Then a structured report is available showing per-file status
- And report can be exported as JSON or text

AT-C5: Move semantics
- Given a cross-volume move operation
- When the copy phase succeeds
- Then source is deleted only after destination is confirmed
- When user cancels mid-transfer
- Then source files for incomplete transfers are NOT deleted

## D) TUI
AT-D1: Key-driven navigation
- When moving selection, opening directories, and going back/forward
- Then state updates instantly and no rendering glitches occur

AT-D2: Dual-pane mode
- When in dual-pane mode
- Then each pane has independent navigation and selection
- And Tab switches between panes
- And copy/move operates from active pane to other pane

AT-D3: Transfers screen
- When a transfer runs
- Then a queue view shows state and progress and supports cancel

AT-D4: Conflict resolution prompt
- When a conflict occurs during transfer (Ask mode)
- Then a bottom-bar modal appears with single-keypress options
- And "apply to all" options work correctly

AT-D5: Hidden files toggle
- When toggling hidden file visibility
- Then hidden/system files appear or disappear accordingly

## E) GUI (Tauri v2 + React + Bun)
AT-E1: Command invocation
- When the GUI requests a directory listing
- Then it calls Rust using `invoke()` and renders results. [web:119]

AT-E2: Event subscription
- When a transfer starts
- Then the GUI subscribes via `listen()` and updates progress UI in real time. [web:114]

AT-E3: Bun-only CI
- Given a clean checkout
- When running `bun install` and `bun test`
- Then frontend dependencies install and tests run using Bun tooling. [web:105][web:107]

AT-E4: Dual-pane mode
- When enabling dual-pane mode
- Then two independent file lists are displayed side-by-side
- And operations can target the other pane

AT-E5: Drag & Drop (internal)
- When dragging files from one pane to another
- Then a copy/move operation is initiated correctly

AT-E6: Drag & Drop (external)
- When dropping files from Windows Explorer onto ZManager
- Then a copy/move operation is initiated for the dropped files
- When dragging files from ZManager to Windows Explorer
- Then files are exported correctly

AT-E7: Context menus
- When right-clicking a file
- Then a context menu appears with expected actions
- When right-clicking empty space
- Then a background context menu appears with folder-level actions

AT-E8: Virtualized list performance
- Given a directory with 50k+ entries
- When rendering the file list
- Then scrolling remains smooth (60fps target)
- And memory usage stays bounded

AT-E9: Open with default app
- When double-clicking a file
- Then it opens with the system default application

AT-E10: Smart address bar
- When clicking a breadcrumb segment
- Then navigation occurs to that directory
- When clicking empty area of address bar
- Then edit mode activates with full path selected
- When typing in edit mode
- Then autocomplete suggestions appear
- When pressing Enter
- Then navigation occurs to typed path
- When pressing Escape
- Then edit mode cancels and breadcrumbs restore

AT-E11: Quick Access sidebar
- When clicking a favorite
- Then navigation occurs to that directory
- When dragging a folder to Quick Access
- Then it is added to favorites
- When right-clicking a favorite and selecting Remove
- Then it is removed from favorites

AT-E12: Toast notifications
- When an operation fails
- Then a toast notification appears
- And auto-dismisses after 5 seconds
- When clicking the toast
- Then error details are shown

## F) File Watching
AT-F1: Live directory updates
- Given a directory is being viewed
- When a file is created/modified/deleted externally
- Then the listing updates automatically (within debounce window ~300ms)

AT-F2: Selection preservation
- When the directory auto-refreshes due to external change
- Then the current selection is preserved where possible

AT-F3: Watch error handling
- Given a network drive or inaccessible directory
- When watching fails
- Then a graceful fallback to manual refresh occurs
- And error is logged

## G) Regression/performance
AT-G1: Benchmarks recorded
- Core directory enumeration and sorting have benchmark baselines
- Any regression > X% fails CI (threshold to be decided)

AT-G2: Crash-free smoke
- 30-minute randomized operation sequence (browse/search/copy/cancel/delete)
- No crashes; errors are surfaced and logged with context.

## H) Clipboard Integration
AT-H1: Copy to clipboard
- When copying files in ZManager
- Then file paths are placed in Windows clipboard (CF_HDROP)
- And can be pasted in Windows Explorer

AT-H2: Paste from clipboard
- When copying files in Windows Explorer
- And pasting in ZManager
- Then a copy operation is initiated correctly

AT-H3: Cut and paste
- When cutting files in ZManager
- And pasting in another location
- Then a move operation is initiated
- And cut files appear dimmed until paste

## I) Quick Access / Favorites
AT-I1: Add favorite (TUI)
- When pressing the "add favorite" keybinding on a directory
- Then it is added to Quick Access

AT-I2: Add favorite (GUI)
- When right-clicking a folder and selecting "Add to Quick Access"
- Then it is added to the sidebar

AT-I3: Navigate via favorite
- When selecting a favorite
- Then navigation occurs to that directory

AT-I4: Broken favorite handling
- Given a favorite points to a non-existent path
- When viewing Quick Access
- Then the favorite is marked as broken/unavailable

## J) Configuration
AT-J1: Config file loading
- Given a custom config.toml exists
- When starting ZManager
- Then custom settings are applied

AT-J2: Default config creation
- Given no config file exists
- When starting ZManager
- Then a default config.toml is created

AT-J3: Keybinding customization
- Given custom keybindings in config
- When using the application
- Then custom keybindings work as configured

## K) Crash Reporting
AT-K1: Crash capture
- Given a panic occurs
- When the application crashes
- Then a crash dump file is written

AT-K2: Crash detection on startup
- Given a crash dump exists from previous run
- When starting ZManager
- Then user is notified and offered to view/export the report

## L) Additional Core Operations
AT-L1: Create folder
- When creating a new folder
- Then the folder is created and selected
- When creating with a name that exists
- Then an error is shown and no data is lost

AT-L2: Properties view
- When selecting a file and viewing properties
- Then name, size, dates, and attributes are displayed correctly
- When selecting a folder
- Then item count and total size are calculated

AT-L3: Drive navigation
- When viewing drives list
- Then all available drives are shown with free space
- When clicking a drive
- Then navigation occurs to drive root

AT-L4: Empty folder display
- Given a folder contains no files
- When viewing that folder
- Then "This folder is empty" is displayed
- And drag-drop still works

AT-L5: Directory access error
- Given a folder with no read permission
- When attempting to view it
- Then a clear error message is shown
- And navigation to parent is offered

AT-L6: Startup behavior
- Given ZManager was previously closed with directories open
- When launching ZManager
- Then the same directories are restored in each pane
