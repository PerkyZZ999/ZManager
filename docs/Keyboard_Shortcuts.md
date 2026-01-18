# ZManager — Default Keyboard Shortcuts

This document defines the default keybindings for both TUI and GUI. All keybindings are configurable via `config.toml`.

## Design Principles
- **Intuitive**: Common operations use familiar shortcuts (Ctrl+C, Ctrl+V, etc.)
- **Efficient**: Power users can navigate entirely by keyboard
- **Consistent**: Same actions use same keys across TUI and GUI where possible
- **Discoverable**: Common shortcuts match Windows Explorer conventions

---

## Navigation

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Move selection down | `j` / `↓` | `↓` | Vim-style `j` in TUI |
| Move selection up | `k` / `↑` | `↑` | Vim-style `k` in TUI |
| Move selection to top | `g` `g` / `Home` | `Home` | Double-tap `g` in TUI |
| Move selection to bottom | `G` / `End` | `End` | Shift+g in TUI |
| Page down | `Ctrl+d` / `PageDown` | `PageDown` | Half-page in TUI |
| Page up | `Ctrl+u` / `PageUp` | `PageUp` | Half-page in TUI |
| Enter directory / Open file | `Enter` / `l` | `Enter` / double-click | Vim-style `l` in TUI |
| Go to parent directory | `h` / `Backspace` | `Backspace` / `Alt+↑` | Vim-style `h` in TUI |
| Go back (history) | `Alt+←` / `[` | `Alt+←` | Browser-style |
| Go forward (history) | `Alt+→` / `]` | `Alt+→` | Browser-style |
| Go to home directory | `~` | `Alt+Home` | |
| Go to root | `/` (then clear) | `Ctrl+\` | |
| Focus address bar | `:` | `Ctrl+L` / `F4` | Command mode in TUI |

---

## Selection

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Toggle selection | `Space` | `Space` / `Ctrl+Click` | |
| Select all | `Ctrl+a` | `Ctrl+A` | |
| Deselect all | `Escape` | `Escape` | |
| Extend selection down | `Shift+j` / `Shift+↓` | `Shift+↓` | |
| Extend selection up | `Shift+k` / `Shift+↑` | `Shift+↑` | |
| Select range | `Shift+Space` | `Shift+Click` | From anchor to cursor |
| Invert selection | `*` | `Ctrl+I` | |

---

## File Operations

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Copy | `c` / `Ctrl+c` | `Ctrl+C` | To clipboard |
| Cut | `x` / `Ctrl+x` | `Ctrl+X` | To clipboard |
| Paste | `p` / `Ctrl+v` | `Ctrl+V` | From clipboard |
| Delete (to Recycle Bin) | `d` / `Delete` | `Delete` | Default safe delete |
| Delete permanently | `Shift+d` / `Shift+Delete` | `Shift+Delete` | Bypass Recycle Bin |
| Rename | `r` / `F2` | `F2` | Inline rename |
| New folder | `n` / `Ctrl+Shift+n` | `Ctrl+Shift+N` | |
| New file | `Ctrl+n` | `Ctrl+N` | Optional v1.5 |
| Open with default app | `Enter` (on file) | `Enter` / double-click | |
| Open with... | `Shift+Enter` | `Shift+Enter` | Choose application |
| Properties | `Alt+Enter` / `i` | `Alt+Enter` | |
| Refresh | `Ctrl+r` / `F5` | `F5` / `Ctrl+R` | |

---

## Dual-Pane

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Switch active pane | `Tab` | `Tab` | |
| Toggle dual-pane mode | `Ctrl+\` | `Ctrl+\` | |
| Copy to other pane | `C` (Shift+c) | `F5` | Traditional file manager |
| Move to other pane | `M` (Shift+m) | `F6` | Traditional file manager |
| Swap panes | `Ctrl+Tab` | `Ctrl+Tab` | |
| Sync panes (same dir) | `=` | `Ctrl+=` | |

---

## Search & Filter

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Quick filter (type-ahead) | Just start typing | Just start typing | Incremental filter |
| Search prompt | `/` | `Ctrl+F` | Full search |
| Clear filter | `Escape` | `Escape` | |
| Find next | `n` | `F3` / `Enter` | |
| Find previous | `N` (Shift+n) | `Shift+F3` | |

---

## View & Display

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Toggle hidden files | `.` / `Ctrl+h` | `Ctrl+H` | |
| Sort by name | `s` `n` | (column click) | |
| Sort by size | `s` `s` | (column click) | |
| Sort by date | `s` `d` | (column click) | |
| Sort by extension | `s` `e` | (column click) | |
| Reverse sort order | `s` `r` | (column click) | |
| Toggle details/compact | `v` | `Ctrl+Shift+V` | |

---

## Quick Access / Favorites

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Go to favorite 1-9 | `1` - `9` | `Ctrl+1` - `Ctrl+9` | Quick jump |
| Add current dir to favorites | `Ctrl+d` | `Ctrl+D` | |
| Show/toggle favorites panel | `Ctrl+b` | `Ctrl+B` | Sidebar |

---

## Transfers

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Show transfers view | `t` | `Ctrl+T` | Queue panel |
| Pause selected job | `P` | (button) | |
| Resume selected job | `R` | (button) | |
| Cancel selected job | `X` | (button) | |

---

## Conflict Resolution (TUI modal)

| Key | Action |
|-----|--------|
| `o` | Overwrite this file |
| `s` | Skip this file |
| `r` | Rename (auto-generate new name) |
| `a` | Overwrite ALL remaining conflicts |
| `n` | Skip ALL remaining conflicts (None) |
| `c` | Cancel entire operation |

---

## Application

| Action | TUI | GUI | Notes |
|--------|-----|-----|-------|
| Quit | `q` / `Ctrl+q` | `Alt+F4` | |
| Help | `?` / `F1` | `F1` | |
| Command palette | `:` | `Ctrl+Shift+P` | |
| Settings | (edit config.toml) | `Ctrl+,` | |

---

## Customization

Keybindings can be customized in `%APPDATA%\ZManager\config.toml`:

```toml
[keybindings.tui]
# Action = "key" or ["key1", "key2"] for multiple bindings
navigate_down = ["j", "Down"]
navigate_up = ["k", "Up"]
enter = ["Enter", "l"]
go_up = ["h", "Backspace"]
toggle_hidden = "."
delete = "d"
delete_permanent = "D"
# ... etc

[keybindings.gui]
# GUI uses standard modifier notation: Ctrl+, Shift+, Alt+
copy = "Ctrl+C"
paste = "Ctrl+V"
delete = "Delete"
delete_permanent = "Shift+Delete"
# ... etc
```

---

## Notes

- **TUI vim-style**: The TUI supports vim-like navigation (`h/j/k/l`) by default but also supports arrow keys
- **GUI Windows-style**: The GUI follows Windows Explorer conventions for familiarity
- **Modifiers**: `Ctrl`, `Shift`, `Alt` work as expected; `Ctrl+Shift+` for advanced actions
- **Escape**: Universal "cancel/clear/deselect" action
- **Enter**: Universal "confirm/open/execute" action
