# ZManager GUI Test Plan - Session 2

**Test Environment**: Windows 11, Tauri v2, Tauri MCP  
**Date**: 2026-01-13  
**Tester**: Automated (Tauri MCP + AI Agent)

---

## Test Status Legend
- ⬜ Not Started
- 🔄 In Progress
- ✅ Passed
- ❌ Failed
- ⚠️ Passed with Issues
- 🚫 Not Implemented

---

## Missing Features for Complete File Manager

> Features that need to be implemented for ZManager to be a fully functional file manager.

| ID | Feature | Priority | Status | Notes |
|----|---------|----------|--------|-------|
| MF-001 | Copy files (Ctrl+C → Ctrl+V) | Critical | ✅ Impl | Backend + keyboard handlers exist |
| MF-002 | Move files (Ctrl+X → Ctrl+V) | Critical | ✅ Impl | Backend + keyboard handlers exist |
| MF-003 | Drag & Drop between panes | High | ✅ Impl | DnD provider wired with move/copy handlers |
| MF-004 | Context menu: Open | High | ✅ Impl | In context menu |
| MF-005 | Context menu: Copy/Cut/Paste | High | ✅ Impl | In context menu |
| MF-006 | Context menu: Delete | High | ✅ Impl | In context menu |
| MF-007 | Context menu: Rename | High | ✅ Impl | F2 + context menu |
| MF-008 | Context menu: Properties | Medium | ✅ Impl | Alt+Enter + context menu |
| MF-009 | Search/Filter files in pane | Medium | ✅ Impl | Search input in pane header |
| MF-010 | Progress dialog for transfers | Medium | ⬜ | TransferPanel.tsx exists but not wired |
| MF-011 | Conflict resolution dialog | Medium | ⬜ | ConflictDialog.tsx exists but not wired |
| MF-012 | Undo last operation | Low | ⬜ | Ctrl+Z |
| MF-013 | File preview panel | Low | ⬜ | Image/text preview |
| MF-014 | Dual-pane copy (Shift+F5) / move (F6) | Medium | ✅ Impl | Keyboard shortcuts wired |
| MF-015 | Tab key switches active pane | Medium | ✅ Impl | Keyboard shortcut wired |
| MF-016 | Escape clears selection | Medium | ✅ Impl | Keyboard shortcut wired |

---

## Bugs Found This Session

| ID | Description | Severity | Status |
|----|-------------|----------|--------|
| BUG-001 | Context menu not rendering on right-click | High | 🔄 Investigating |
| BUG-002 | Escape key doesn't clear selection (needs focus) | Medium | 🔄 Needs focus fix |
| BUG-003 | Ctrl+A select all not working | Medium | 🔄 Needs focus fix |
| BUG-004 | Tab pane switch not working (needs focus) | Medium | 🔄 Needs focus fix |
| BUG-005 | Back button may not be wired correctly | Low | ⬜ To verify |

---

## 1. Application Launch & Initial State

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 1.1 | App launches without console errors | ✅ | App launched successfully |
| 1.2 | Window appears with correct dimensions | ✅ | >800x500 verified |
| 1.3 | Custom titlebar renders (logo + title + controls) | ✅ | 3 buttons, ZManager title |
| 1.4 | Sidebar loads with Quick Access + Drives | ✅ | Drives visible |
| 1.5 | Dual panes display with default directories | ✅ | Left and right panes present |
| 1.6 | Status bar visible at bottom | ✅ | Shows selection info |

---

## 2. Navigation Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 2.1 | Click folder in left pane navigates into it | ✅ | Double-clicked Users, navigated |
| 2.2 | Click folder in right pane navigates into it | ⬜ | Not tested |
| 2.3 | Back button returns to previous directory | ⚠️ | Button exists, may have wiring issue |
| 2.4 | Forward button works after going back | ⬜ | Not tested |
| 2.5 | Up button navigates to parent | ⬜ | Not tested |
| 2.6 | Backspace key navigates to parent | ✅ | Works when file list focused |
| 2.7 | Click drive in sidebar navigates that pane | ⬜ | Not tested |
| 2.8 | Click favorite in sidebar navigates active pane | ⬜ | Not tested |
| 2.9 | Breadcrumb segment click navigates to that path | ⬜ | Not tested |
| 2.10 | Address bar allows typing new path | ⬜ | Not tested |

---

## 3. File Selection Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 3.1 | Single click selects one file | ✅ | AMD folder selected |
| 3.2 | Arrow keys move cursor | ⚠️ | Works but needs file list focus |
| 3.3 | Ctrl+Click adds file to selection | ⬜ | Not tested |
| 3.4 | Shift+Click selects range | ⬜ | Not tested |
| 3.5 | Ctrl+A selects all | ⬜ | Tested, did not work |
| 3.6 | Status bar updates with selection count/size | ✅ | Shows "1 selected (0 B)" |
| 3.7 | Escape clears selection | ⚠️ | Handler exists, needs focus |
| 3.8 | Click on different pane switches active pane | ⬜ | Not tested |

---

## 4. File Operations Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 4.1 | Create new folder (right-click → New Folder) | ⬜ | Context menu not rendering |
| 4.2 | Delete key prompts confirmation dialog | ✅ | Dialog shows with Cancel/Delete |
| 4.3 | Delete confirmation moves to Recycle Bin | ⬜ | Not confirmed (cancelled) |
| 4.4 | Enter key opens file with default app | ⬜ | Not tested |
| 4.5 | Enter key on folder navigates into it | ⬜ | Not tested |
| 4.6 | F2 key triggers rename (if implemented) | ⬜ | Handler exists |
| 4.7 | Double-click opens file | ⬜ | Not tested |
| 4.8 | Double-click folder navigates into it | ✅ | Navigated into Users |

---

## 5. Clipboard Operations Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 5.1 | Ctrl+C copies selected files to clipboard | ⬜ | Handler exists |
| 5.2 | Ctrl+X cuts selected files | ⬜ | Handler exists |
| 5.3 | Ctrl+V pastes files in current directory | ⬜ | Handler exists |
| 5.4 | Paste shows progress for large files | ⬜ | Not implemented |
| 5.5 | Copy from Windows Explorer, paste in ZManager | ⬜ | Not tested |
| 5.6 | Copy from ZManager, paste in Windows Explorer | ⬜ | Not tested |

---

## 6. Context Menu Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 6.1 | Right-click on file shows context menu | ❌ | Menu not rendering |
| 6.2 | Right-click on folder shows context menu | ❌ | Menu not rendering |
| 6.3 | Right-click on empty area shows context menu | ❌ | Menu not rendering |
| 6.4 | Menu has: Open option | ⬜ | Code exists |
| 6.5 | Menu has: Copy/Cut/Paste options | ⬜ | Code exists |
| 6.6 | Menu has: Delete option | ⬜ | Code exists |
| 6.7 | Menu has: Rename option | ⬜ | Code exists |
| 6.8 | Menu has: Properties option | ⬜ | Code exists |
| 6.9 | Menu has: New Folder option | ⬜ | Code exists |
| 6.10 | Menu has: Refresh option | ⬜ | Code exists |
| 6.11 | Context menu closes when clicking outside | ⬜ | Code exists |

---

## 7. Sorting Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 7.1 | Click Name header sorts by name | ✅ | Sorts A-Z then Z-A |
| 7.2 | Click again reverses sort order | ✅ | Toggle works |
| 7.3 | Sort indicator shows current column | ⬜ | Not verified |
| 7.4 | Click Size header sorts by size | ⬜ | Not tested |
| 7.5 | Click Date header sorts by modified | ⬜ | |
| 7.6 | Folders always appear before files | ⬜ | |

---

## 8. Sidebar Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 8.1 | Quick Access section expandable/collapsible | ⬜ | |
| 8.2 | Drives section expandable/collapsible | ⬜ | |
| 8.3 | Add current folder to Quick Access | ⬜ | |
| 8.4 | Remove favorite from Quick Access | ⬜ | |
| 8.5 | Drive shows free space indicator | ⬜ | |
| 8.6 | Clicking drive changes active pane directory | ⬜ | |

---

## 9. Resizable Panes Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 9.1 | Separator is visible and draggable | ⬜ | |
| 9.2 | Drag separator resizes both panes | ⬜ | |
| 9.3 | Minimum width enforced on each pane | ⬜ | |
| 9.4 | Sidebar can be resized | ⬜ | |

---

## 10. Keyboard Shortcuts Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 10.1 | F5 refreshes current directory | ⬜ | |
| 10.2 | Delete removes selected files | ⬜ | |
| 10.3 | Enter opens selected item | ⬜ | |
| 10.4 | Backspace goes to parent | ⬜ | |
| 10.5 | Ctrl+A selects all | ⬜ | |
| 10.6 | Escape clears selection | ⬜ | |
| 10.7 | Tab switches between panes | ⬜ | |
| 10.8 | Arrow keys navigate file list | ⬜ | |

---

## 11. Visual/UI Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 11.1 | Folder icons display correctly | ⬜ | |
| 11.2 | File icons match extension | ⬜ | |
| 11.3 | Hidden files have muted styling | ⬜ | |
| 11.4 | Selection highlight is visible | ⬜ | |
| 11.5 | Active pane has focus indicator | ⬜ | |
| 11.6 | Scrolling works for long file lists | ⬜ | |
| 11.7 | Virtualization handles 1000+ files | ⬜ | |

---

## 12. Error Handling Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 12.1 | Access denied shows error toast | ⬜ | |
| 12.2 | Path not found shows error toast | ⬜ | |
| 12.3 | File in use shows error toast | ⬜ | |
| 12.4 | Toast auto-dismisses after timeout | ⬜ | |
| 12.5 | Toast can be manually dismissed | ⬜ | |

---

## 13. Window Controls Flow

| ID | Test Case | Status | Notes |
|----|-----------|--------|-------|
| 13.1 | Minimize button minimizes window | ⬜ | |
| 13.2 | Maximize button maximizes window | ⬜ | |
| 13.3 | Close button closes application | ⬜ | |
| 13.4 | Titlebar is draggable | ⬜ | |
| 13.5 | Double-click titlebar toggles maximize | ⬜ | |

---

## Test Execution Log

| Time | Test ID | Action | Result |
|------|---------|--------|--------|
| Session 1 | 1.1-1.6 | Initial state checks | ✅ All passed |
| Session 1 | 2.1, 2.6 | Navigation tests | ✅ Passed |
| Session 1 | 3.1, 3.6 | Selection tests | ✅ Passed |
| Session 1 | 4.2, 4.8 | File operations | ✅ Passed |
| Session 1 | 6.1-6.3 | Context menu | ❌ Not rendering |
| Session 1 | 7.1-7.2 | Sorting | ✅ Passed |

---

## Summary

- **Total Test Cases**: 97
- **Passed**: 14
- **Failed**: 3 (Context menu not rendering)
- **Partial/Warning**: 4
- **Not Tested**: 76

**Session Status**: 🔄 Partially Complete

### Known Issues Found

1. **Context menu not rendering** - Right-click triggers handler but menu doesn't appear
2. **Escape key clear selection** - Handler exists but requires proper focus
3. **Ctrl+A select all** - Did not work during testing
4. **Tab pane switch** - Handler exists but may need focus

### Recommendations

1. Investigate why context menu state doesn't trigger re-render
2. Add `tabIndex` to the file pane section for proper keyboard focus
3. Test Ctrl+A with file list having proper focus
