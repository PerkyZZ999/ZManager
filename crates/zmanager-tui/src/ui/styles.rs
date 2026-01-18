//! Color and style definitions.

use ratatui::style::{Color, Modifier, Style};

/// Application color scheme and styles.
pub struct Styles;

impl Styles {
    // === Colors ===

    /// Background color.
    pub const BG: Color = Color::Reset;

    /// Primary foreground color.
    pub const FG: Color = Color::White;

    /// Accent color for highlights.
    pub const ACCENT: Color = Color::Cyan;

    /// Secondary accent.
    pub const ACCENT_DIM: Color = Color::DarkGray;

    /// Directory color.
    pub const DIR: Color = Color::Blue;

    /// Executable/program color.
    pub const EXE: Color = Color::Green;

    /// Archive color.
    pub const ARCHIVE: Color = Color::Red;

    /// Image color.
    pub const IMAGE: Color = Color::Magenta;

    /// Hidden file color.
    pub const HIDDEN: Color = Color::DarkGray;

    /// Error color.
    pub const ERROR: Color = Color::Red;

    /// Warning color.
    pub const WARNING: Color = Color::Yellow;

    /// Success color.
    pub const SUCCESS: Color = Color::Green;

    /// Selection background.
    pub const SELECTION_BG: Color = Color::DarkGray;

    /// Cursor background.
    pub const CURSOR_BG: Color = Color::Rgb(50, 50, 80);

    // === Styles ===

    /// Normal text style.
    pub fn normal() -> Style {
        Style::default().fg(Self::FG)
    }

    /// Header style.
    pub fn header() -> Style {
        Style::default().fg(Self::ACCENT).add_modifier(Modifier::BOLD)
    }

    /// Directory entry style.
    pub fn directory() -> Style {
        Style::default().fg(Self::DIR).add_modifier(Modifier::BOLD)
    }

    /// Executable file style.
    pub fn executable() -> Style {
        Style::default().fg(Self::EXE)
    }

    /// Archive file style.
    pub fn archive() -> Style {
        Style::default().fg(Self::ARCHIVE)
    }

    /// Image file style.
    pub fn image() -> Style {
        Style::default().fg(Self::IMAGE)
    }

    /// Hidden file style.
    pub fn hidden() -> Style {
        Style::default().fg(Self::HIDDEN)
    }

    /// Selected item style.
    pub fn selected() -> Style {
        Style::default().bg(Self::SELECTION_BG).add_modifier(Modifier::BOLD)
    }

    /// Cursor (focused) item style.
    pub fn cursor() -> Style {
        Style::default().bg(Self::CURSOR_BG)
    }

    /// Cursor + selected style.
    pub fn cursor_selected() -> Style {
        Style::default()
            .bg(Self::CURSOR_BG)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }

    /// Status bar style.
    pub fn status_bar() -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }

    /// Active pane border.
    pub fn active_border() -> Style {
        Style::default().fg(Self::ACCENT)
    }

    /// Inactive pane border.
    pub fn inactive_border() -> Style {
        Style::default().fg(Self::ACCENT_DIM)
    }

    /// Error message style.
    pub fn error() -> Style {
        Style::default().fg(Self::ERROR).add_modifier(Modifier::BOLD)
    }

    /// Warning message style.
    pub fn warning() -> Style {
        Style::default().fg(Self::WARNING)
    }

    /// Success message style.
    pub fn success() -> Style {
        Style::default().fg(Self::SUCCESS)
    }

    /// Size column style.
    pub fn size() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// Date column style.
    pub fn date() -> Style {
        Style::default().fg(Color::Gray)
    }

    /// Get style for a file by extension.
    pub fn for_extension(ext: &str) -> Style {
        match ext.to_lowercase().as_str() {
            // Executables
            "exe" | "bat" | "cmd" | "ps1" | "msi" => Self::executable(),
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => Self::archive(),
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => Self::image(),
            // Default
            _ => Self::normal(),
        }
    }
}
