# ZManager

A fast, dual-pane file manager for Windows, built with Rust.

## Features

- 🚀 **Performance-first**: Native Windows file operations with CopyFileEx
- 📁 **Dual-pane**: Side-by-side directory browsing
- 🖥️ **Dual-frontend**: TUI (terminal) and GUI (Tauri + React)
- ⌨️ **Keyboard-driven**: Full keyboard navigation with Vim-style bindings
- 📋 **Clipboard integration**: Works seamlessly with Windows Explorer
- 🔍 **Fast filtering**: Real-time search and filter

## Project Structure

```
ZManager/
├── crates/
│   ├── zmanager-core/        # Core library (types, sorting, filtering)
│   ├── zmanager-transfer-win/ # Windows file transfer engine (CopyFileEx)
│   ├── zmanager-tui/         # Terminal UI (Ratatui)
│   └── zmanager-tauri/       # GUI backend (Tauri v2)
├── gui/                      # React frontend (coming soon)
└── docs/                     # Documentation
```

## Building

### Prerequisites

- Rust 1.85+ (2024 edition)
- Windows 10/11

### Build Commands

```bash
# Build all crates
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run TUI
cargo run -p zmanager-tui

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## Development Status

🚧 **Currently in Sprint 1**: Project scaffolding and core domain types.

See [Sprint Roadmap](docs/Sprint_Roadmap.md) for the full development plan.

## License

MIT License - see [LICENSE](LICENSE) for details.
