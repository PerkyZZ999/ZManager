# ZManager
<img width="128" height="128" alt="ZManager_Icon" src="https://github.com/user-attachments/assets/996592ae-0375-4d5b-82f0-e663e22af900" />

A fast, dual-pane file manager for Windows, built in Rust with TUI and GUI interfaces.

<p align="center">
  <img src="docs/screenshots/preview.png" alt="ZManager Preview" width="800">
</p>

## Features

- 🚀 **Performance-first**: Native Windows file operations via `CopyFileExW` with real-time progress
- 📁 **Dual-pane**: Side-by-side directory browsing with synchronized operations
- 🖥️ **Dual-frontend**: Terminal (Ratatui) and GUI (Tauri v2 + React 19)
- ⌨️ **Keyboard-driven**: Full keyboard navigation with configurable bindings
- 📋 **Windows clipboard**: Cut/copy/paste files works with Windows Explorer
- 🔍 **Real-time filtering**: Instant search with glob patterns
- 📊 **Transfer engine**: Queued operations with conflict resolution, pause/resume, and progress callbacks
- 🔗 **Symlink support**: Full detection of junctions, symlinks, and reparse points
- 📏 **Long path support**: Automatic `\\?\` prefix for paths ≥240 chars

## Architecture

```
ZManager/
├── crates/
│   ├── zmanager-core/         # Platform-agnostic core (domain types, business logic)
│   ├── zmanager-transfer-win/ # Windows transfer engine (CopyFileExW, clipboard)
│   ├── zmanager-tui/          # Terminal UI (Ratatui + Crossterm)
│   └── zmanager-tauri/        # GUI backend (Tauri v2) + React frontend
└── docs/                      # Documentation & roadmap
```

**Core principle**: All business logic lives in `zmanager-core`. Frontends are thin presentation layers. Transfer engine is Windows-specific for future cross-platform support.

## Keyboard Shortcuts

| Action | Key |
|--------|-----|
| Navigate | `↑`/`↓` or `j`/`k` |
| Enter directory | `Enter` |
| Go up | `Backspace` or `h` |
| Switch pane | `Tab` |
| Toggle select | `Space` |
| Select all | `Ctrl+A` |
| Copy | `Ctrl+C` or `F5` |
| Move | `Ctrl+X` or `F6` |
| Paste | `Ctrl+V` |
| Delete | `Delete` or `F8` |
| New folder | `F7` |
| Rename | `F2` |
| Filter | `/` |
| Refresh | `Ctrl+R` |
| Quit | `q` or `Ctrl+Q` |

See [Keyboard Shortcuts](docs/Keyboard_Shortcuts.md) for the full reference.

## Building

### Prerequisites

- Rust 1.85+ (2024 edition)
- Windows 10/11
- Bun (for GUI frontend)

### Build Commands

```bash
# Build all Rust crates
cargo build --release

# Run tests (257 tests)
cargo test --workspace

# Run TUI
cargo run -p zmanager-tui --release

# GUI development (from crates/zmanager-tauri/)
cd crates/zmanager-tauri
cargo tauri dev

# GUI frontend (from crates/zmanager-tauri/gui/)
bun install
bun run dev
```

### Performance

| Operation | Throughput |
|-----------|------------|
| Directory listing | ~1M files/sec |
| File copy (SSD) | 500+ MB/s |
| 50K file listing | <100ms |

## Development Status

✅ **Phase 5: Polish & Release** (Sprints 17-18)

| Phase | Status |
|-------|--------|
| Phase 1: Core Foundation | ✅ Complete |
| Phase 2: Transfer Engine | ✅ Complete |
| Phase 3: TUI | ✅ Complete |
| Phase 4: GUI | ✅ Complete |
| Phase 5: Polish & Release | 🔄 In Progress |

**Test coverage**: 257 tests (129 core + 72 transfer + 46 TUI + integration)

See [Sprint Roadmap](docs/Sprint_Roadmap.md) for the full development plan.

## Documentation

- [IPC Contract](docs/IPC_Contract.md) - Tauri command specifications
- [Keyboard Shortcuts](docs/Keyboard_Shortcuts.md) - Full keybinding reference
- [Acceptance Tests](docs/Acceptance_Tests.md) - Manual test procedures
- [PRD](docs/PRD_ZManager.md) - Product requirements document

## License

MIT License - see [LICENSE](LICENSE) for details.



