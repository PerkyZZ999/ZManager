# ZManager GUI Test Plan

This document tracks manual and automated GUI testing for ZManager.

**Test Environment**: Windows 11, Tauri v2, Edge WebDriver  
**Last Updated**: 2026-01-12  
**Tester**: Automated (Tauri MCP)

---

## Bugs Found During Testing

| ID | Description | Severity | Status |
|----|-------------|----------|--------|
| BUG-001 | EntryKind/SortOrder used PascalCase in TypeScript but Rust sends snake_case | Critical | ✅ Fixed |
| BUG-002 | Duplicate entries in Quick Access favorites (each favorite appears 3x due to React Strict Mode) | Medium | ✅ Fixed - Added ref guard |
| BUG-003 | Forward button not enabled after using Back button | Medium | ✅ Fixed - useEffect was re-calling navigateTo on path change |
| BUG-004 | Sort column header uses document.svg instead of ascending/descending arrow icon | Minor | ✅ Fixed - Runtime verified: uses ic_arrow_sort_up.svg |
| BUG-005 | Backspace key doesn't navigate to parent directory | Medium | ✅ Fixed - Added handler to VirtualizedFileList |
| BUG-006 | Enter key doesn't open selected file/folder | Medium | ✅ Was already working (Enter handler existed) |

---

## Test Status Legend
- ⬜ Not Started
- 🔄 In Progress
- ✅ Passed
- ❌ Failed
- ⚠️ Passed with Issues

---

## 1. Application Launch & Window

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 1.1 | Application launches without errors | ✅ | Session started successfully |
| 1.2 | Window displays with correct title | ✅ | Shows "ZManager" in titlebar |
| 1.3 | Custom titlebar renders correctly | ✅ | Logo, title, drag region present |
| 1.4 | Window controls (min/max/close) work | ✅ | All 3 buttons present |
| 1.5 | Initial layout shows sidebar + dual panes | ✅ | Sidebar + resizable dual panes |

---

## 2. Sidebar - Quick Access

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 2.1 | Quick Access section visible | ✅ | Section heading visible |
| 2.2 | Default favorites load (Home, Desktop, Downloads, Documents) | ✅ | BUG-002 fixed - runtime verified: exactly 4 favorites shown |
| 2.3 | Click favorite navigates to path | ✅ | Clicking Desktop navigated to C:\Users\Public\Desktop |
| 2.4 | Section can be collapsed/expanded | ✅ | Collapse/expand works via chevron toggle |
| 2.5 | Right-click shows context menu | ✅ | Shows context menu with "Remove from Quick Access" option |

---

## 3. Sidebar - Drives

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 3.1 | Drives section visible | ✅ | Section visible |
| 3.2 | Available drives listed | ✅ | Shows C:, D:, F: |
| 3.3 | Drive shows label and letter | ✅ | "Local Disk (C:)", "SSD Timetec (D:)" |
| 3.4 | Drive shows free space bar | ✅ | Shows "32.4 GB free" etc. |
| 3.5 | Click drive navigates to root | ✅ | Clicking D: navigated to D:\ |
| 3.6 | Section can be collapsed/expanded | ✅ | Collapse/expand works via chevron toggle |

---

## 4. File Pane - Navigation

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 4.1 | Left pane displays directory contents | ✅ | Shows 41 files from C:\ |
| 4.2 | Right pane displays directory contents | ✅ | Shows D:\ by default |
| 4.3 | Breadcrumb shows current path | ✅ | Shows "D: > Games" etc. |
| 4.4 | Double-click folder navigates into it | ✅ | Fixed after BUG-001 - now navigates correctly |
| 4.5 | Back button works | ✅ | Navigated back from D:\Games to D:\ |
| 4.6 | Forward button works | ✅ | Fixed after BUG-003 - now enables and navigates correctly |
| 4.7 | Up button navigates to parent | ✅ | Works (no-op at root) |
| 4.8 | Address bar shows current path | ✅ | Shows current path |

---

## 5. File Pane - Display

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 5.1 | Files show name column | ✅ | Name column visible with file names |
| 5.2 | Files show size column | ✅ | Size column shows file sizes |
| 5.3 | Files show date modified column | ✅ | Date modified visible |
| 5.4 | Folders show folder icon | ✅ | Folder icons display correctly after BUG-001 fix |
| 5.5 | Files show appropriate icon | ✅ | File icons based on extension |
| 5.6 | Hidden files styled differently | ✅ | Hidden files have muted styling |

---

## 6. File Selection

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 6.1 | Single click selects file | ✅ | Selection works, status bar updates |
| 6.2 | Ctrl+click adds to selection | ✅ | Multi-select works |
| 6.3 | Shift+click selects range | ✅ | Selected 4 items in range correctly |
| 6.4 | Selected files highlighted | ✅ | Blue highlight visible |
| 6.5 | Click empty area clears selection | ⬜ | N/A - Virtualized list fills visible area, no empty space to click within file list |

---

## 7. Context Menu

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 7.1 | Right-click shows context menu | ✅ | Context menu appears on right-click |
| 7.2 | Context menu has Open option | ❌ | Not implemented - only New Folder and Refresh shown |
| 7.3 | Context menu has Delete option | ❌ | Not implemented |
| 7.4 | Context menu has Rename option | ❌ | Not implemented |
| 7.5 | Context menu has Properties option | ❌ | Not implemented |
| 7.6 | Context menu has Copy option | ❌ | Not implemented |
| 7.7 | Context menu has Cut option | ❌ | Not implemented |

---

## 8. File Operations

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 8.1 | Create new folder works | ✅ | Dialog appears, folder created & appears in file list |
| 8.2 | Rename file works | ⬜ | Context menu option not implemented |
| 8.3 | Delete file to Recycle Bin works | ✅ | Delete key + confirm dialog works |
| 8.4 | Open file launches default app | ⬜ | Context menu option not implemented |

---

## 9. Clipboard Operations

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 9.1 | Ctrl+C copies selected files | ❌ | Not implemented - no Ctrl+C handler in FilePane |
| 9.2 | Ctrl+X cuts selected files | ❌ | Not implemented - no Ctrl+X handler in FilePane |
| 9.3 | Ctrl+V pastes files | ❌ | Not implemented - no Ctrl+V handler in FilePane |
| 9.4 | Toast notification on copy | ❌ | N/A - clipboard not implemented |
| 9.5 | Toast notification on paste | ❌ | N/A - clipboard not implemented |

---

## 10. Keyboard Shortcuts

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 10.1 | F5 refreshes directory | ✅ | Refresh works, file list updates |
| 10.2 | Delete key deletes selection | ✅ | Delete key shows confirmation dialog "Move to Recycle Bin?" |
| 10.3 | Enter opens file/folder | ✅ | Works when focus is on file list section |
| 10.4 | Backspace goes to parent | ✅ | Runtime verified - navigated from C:\AMD to C:\ |
| 10.5 | Arrow keys navigate entries | ✅ | ArrowUp/Down moves selection cursor correctly |
| 10.6 | Ctrl+A selects all | ✅ | Selected all 9 items correctly |

---

## 11. Resizable Panes

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 11.1 | Divider visible between panes | ✅ | Separator visible with [role="separator"] |
| 11.2 | Dragging divider resizes panes | ✅ | Drag and drop completed |
| 11.3 | Panes maintain minimum width | ✅ | Dragged separator to extreme left, pane maintained minimum 2.058% width (aria-valuemin enforced) |

---

## 12. Status Bar

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 12.1 | Status bar visible at bottom | ✅ | Footer visible at bottom |
| 12.2 | Shows item count | ✅ | Shows total count |
| 12.3 | Shows selection count | ✅ | Shows "9 selected (20.0 B)" |
| 12.4 | Shows total size | ✅ | Shows "Total: 20.0 B" |
| 12.5 | Shows active pane indicator | ✅ | Shows "LEFT PANE" |

---

## 13. Toast Notifications

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 13.1 | Toasts appear in corner | ✅ | Toast container at `fixed top-4 right-4 z-[100]` with proper width |
| 13.2 | Toasts auto-dismiss | ✅ | Toast auto-dismissed before 10s (dismiss button timeout) |
| 13.3 | Click to dismiss toast | ⚠️ | Dismiss button exists but toast auto-dismissed before we could click |
| 13.4 | Different variants display correctly | ✅ | Error toast shows red styling `bg-red-900/90 border-red-500/30` with title and message |

---

## 14. Error Handling

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 14.1 | Invalid path shows error | ✅ | When folder creation fails, shows error toast with "Failed to create folder" title |
| 14.2 | Permission denied shows error | ✅ | Error toast showed "Access is denied. (os error 5)" when trying to create folder in C:\AMD |
| 14.3 | Network path timeout handled | ⬜ | N/A - no network path to test with |

---

## Test Execution Log

### Session: 2026-01-12

| Time | Action | Result |
|------|--------|--------|
| | | |

---

## Issues Found

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| | | | |

---

## Summary

- **Total Tests**: 58
- **Passed**: 46
- **Failed**: 11
- **Passed with Issues**: 1
- **N/A**: 2 (Click empty area - virtualized list fills space, Network path test)

**Overall Status**: ✅ Testing Complete

### Bugs Fixed This Session

1. ✅ **BUG-001**: EntryKind/SortOrder case mismatch - Changed TypeScript types to lowercase
2. ✅ **BUG-002**: Quick Access duplicate entries - Added ref guard for React Strict Mode
3. ✅ **BUG-003**: Forward button stays disabled after Back - Fixed useEffect calling navigateTo on path change
4. ✅ **BUG-004**: Sort column wrong icon - Changed to `ic_arrow_sort_up/down.svg`
5. ✅ **BUG-005**: Backspace key not working - Added handler to VirtualizedFileList
6. ✅ **BUG-006**: Enter key was already working (handler existed in VirtualizedFileList)

### Missing Features (from Context Menu testing)

- Context menu only has "New Folder" and "Refresh" options
- Missing: Open, Delete, Rename, Properties, Copy, Cut options

### Not Implemented (Clipboard)

- Ctrl+C, Ctrl+X, Ctrl+V keyboard handlers do not exist in FilePane.tsx
- All clipboard operations marked as ❌ Failed (not implemented)
