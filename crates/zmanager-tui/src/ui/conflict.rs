//! Conflict resolution modal for file operations.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// Conflict information for display.
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    /// Source file path.
    pub source: PathBuf,
    /// Destination file path.
    pub destination: PathBuf,
    /// Source file size.
    pub source_size: u64,
    /// Destination file size.
    pub dest_size: u64,
    /// Source modification time (formatted).
    pub source_modified: String,
    /// Destination modification time (formatted).
    pub dest_modified: String,
}

/// Conflict resolution options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Overwrite the destination.
    Overwrite,
    /// Skip this file.
    Skip,
    /// Rename the source file.
    Rename,
    /// Keep the larger file.
    KeepLarger,
    /// Keep the newer file.
    KeepNewer,
    /// Cancel the entire operation.
    Cancel,
}

impl ConflictResolution {
    /// Get the hotkey for this resolution.
    pub fn hotkey(&self) -> char {
        match self {
            Self::Overwrite => 'o',
            Self::Skip => 's',
            Self::Rename => 'r',
            Self::KeepLarger => 'l',
            Self::KeepNewer => 'n',
            Self::Cancel => 'c',
        }
    }

    /// Get the label for this resolution.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Overwrite => "Overwrite",
            Self::Skip => "Skip",
            Self::Rename => "Rename",
            Self::KeepLarger => "Keep Larger",
            Self::KeepNewer => "Keep Newer",
            Self::Cancel => "Cancel All",
        }
    }

    /// Get all resolution options.
    pub fn all() -> &'static [ConflictResolution] {
        &[
            Self::Overwrite,
            Self::Skip,
            Self::Rename,
            Self::KeepLarger,
            Self::KeepNewer,
            Self::Cancel,
        ]
    }
}

/// Result of conflict modal interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResult {
    /// Modal is still open.
    Open,
    /// User selected a resolution.
    Resolved(ConflictResolution, bool), // (resolution, apply_to_all)
}

/// Conflict resolution modal.
#[derive(Debug, Clone)]
pub struct ConflictModal {
    /// Conflict information.
    pub info: ConflictInfo,
    /// Whether "apply to all" is selected.
    pub apply_to_all: bool,
    /// Remaining conflicts count.
    pub remaining: usize,
}

impl ConflictModal {
    /// Create a new conflict modal.
    pub fn new(info: ConflictInfo, remaining: usize) -> Self {
        Self {
            info,
            apply_to_all: false,
            remaining,
        }
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) -> ConflictResult {
        match (key.modifiers, key.code) {
            // Resolution hotkeys
            (KeyModifiers::NONE, KeyCode::Char('o')) => {
                ConflictResult::Resolved(ConflictResolution::Overwrite, self.apply_to_all)
            }
            (KeyModifiers::NONE, KeyCode::Char('s')) => {
                ConflictResult::Resolved(ConflictResolution::Skip, self.apply_to_all)
            }
            (KeyModifiers::NONE, KeyCode::Char('r')) => {
                ConflictResult::Resolved(ConflictResolution::Rename, self.apply_to_all)
            }
            (KeyModifiers::NONE, KeyCode::Char('l')) => {
                ConflictResult::Resolved(ConflictResolution::KeepLarger, self.apply_to_all)
            }
            (KeyModifiers::NONE, KeyCode::Char('n')) => {
                ConflictResult::Resolved(ConflictResolution::KeepNewer, self.apply_to_all)
            }
            (KeyModifiers::NONE, KeyCode::Char('c')) | (KeyModifiers::NONE, KeyCode::Esc) => {
                ConflictResult::Resolved(ConflictResolution::Cancel, false)
            }
            // Shift + key for "apply to all"
            (KeyModifiers::SHIFT, KeyCode::Char('O')) => {
                ConflictResult::Resolved(ConflictResolution::Overwrite, true)
            }
            (KeyModifiers::SHIFT, KeyCode::Char('S')) => {
                ConflictResult::Resolved(ConflictResolution::Skip, true)
            }
            (KeyModifiers::SHIFT, KeyCode::Char('R')) => {
                ConflictResult::Resolved(ConflictResolution::Rename, true)
            }
            (KeyModifiers::SHIFT, KeyCode::Char('L')) => {
                ConflictResult::Resolved(ConflictResolution::KeepLarger, true)
            }
            (KeyModifiers::SHIFT, KeyCode::Char('N')) => {
                ConflictResult::Resolved(ConflictResolution::KeepNewer, true)
            }
            // Toggle apply to all with 'a'
            (KeyModifiers::NONE, KeyCode::Char('a')) => {
                self.apply_to_all = !self.apply_to_all;
                ConflictResult::Open
            }
            _ => ConflictResult::Open,
        }
    }

    /// Render the conflict modal.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Calculate modal size (centered, 60x14)
        let width = area.width.clamp(50, 70);
        let height = 14u16;
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let modal_area = Rect::new(x, y, width, height);

        // Clear background
        Clear.render(modal_area, buf);

        // Create block
        let title = format!(" File Conflict ({} remaining) ", self.remaining);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(title);

        let inner = block.inner(modal_area);
        block.render(modal_area, buf);

        // Layout: info section + options
        let chunks = Layout::vertical([
            Constraint::Length(5), // File info
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Options
            Constraint::Length(1), // Apply to all hint
        ])
        .split(inner);

        // File info
        self.render_file_info(chunks[0], buf);

        // Options
        self.render_options(chunks[2], buf);

        // Apply to all hint
        self.render_hint(chunks[3], buf);
    }

    fn render_file_info(&self, area: Rect, buf: &mut Buffer) {
        let source_name = self.info.source.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let lines = vec![
            Line::from(vec![
                Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(truncate(&source_name, 50)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Source:      "),
                Span::styled(format_size(self.info.source_size), Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(&self.info.source_modified, Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw("Destination: "),
                Span::styled(format_size(self.info.dest_size), Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(&self.info.dest_modified, Style::default().fg(Color::Gray)),
            ]),
        ];

        Paragraph::new(lines).render(area, buf);
    }

    fn render_options(&self, area: Rect, buf: &mut Buffer) {
        // First row: O/S/R
        let row1 = Line::from(vec![
            Span::styled("[O]", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
            Span::raw("verwrite  "),
            Span::styled("[S]", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
            Span::raw("kip  "),
            Span::styled("[R]", Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)),
            Span::raw("ename  "),
        ]);

        // Second row: L/N/C
        let row2 = Line::from(vec![
            Span::styled("[L]", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            Span::raw("arger  "),
            Span::styled("[N]", Style::default().add_modifier(Modifier::BOLD).fg(Color::Magenta)),
            Span::raw("ewer  "),
            Span::styled("[C]", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
            Span::raw("ancel  "),
        ]);

        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        Paragraph::new(row1).alignment(Alignment::Center).render(chunks[0], buf);
        Paragraph::new(row2).alignment(Alignment::Center).render(chunks[1], buf);
    }

    fn render_hint(&self, area: Rect, buf: &mut Buffer) {
        let checkbox = if self.apply_to_all { "[✓]" } else { "[ ]" };
        let hint = Line::from(vec![
            Span::styled("[A]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("pply to all {} ", checkbox)),
            Span::styled("(or SHIFT+key)", Style::default().add_modifier(Modifier::DIM)),
        ]);

        Paragraph::new(hint)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_hotkeys() {
        assert_eq!(ConflictResolution::Overwrite.hotkey(), 'o');
        assert_eq!(ConflictResolution::Skip.hotkey(), 's');
        assert_eq!(ConflictResolution::Cancel.hotkey(), 'c');
    }

    #[test]
    fn handle_key_overwrite() {
        let info = ConflictInfo {
            source: PathBuf::from("a.txt"),
            destination: PathBuf::from("b.txt"),
            source_size: 100,
            dest_size: 200,
            source_modified: "2024-01-01".to_string(),
            dest_modified: "2024-01-02".to_string(),
        };
        let mut modal = ConflictModal::new(info, 5);
        
        let result = modal.handle_key(KeyEvent::from(KeyCode::Char('o')));
        assert_eq!(result, ConflictResult::Resolved(ConflictResolution::Overwrite, false));
    }

    #[test]
    fn handle_key_shift_applies_to_all() {
        let info = ConflictInfo {
            source: PathBuf::from("a.txt"),
            destination: PathBuf::from("b.txt"),
            source_size: 100,
            dest_size: 200,
            source_modified: "2024-01-01".to_string(),
            dest_modified: "2024-01-02".to_string(),
        };
        let mut modal = ConflictModal::new(info, 5);
        
        let result = modal.handle_key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT));
        assert_eq!(result, ConflictResult::Resolved(ConflictResolution::Skip, true));
    }

    #[test]
    fn toggle_apply_to_all() {
        let info = ConflictInfo {
            source: PathBuf::from("a.txt"),
            destination: PathBuf::from("b.txt"),
            source_size: 100,
            dest_size: 200,
            source_modified: "2024-01-01".to_string(),
            dest_modified: "2024-01-02".to_string(),
        };
        let mut modal = ConflictModal::new(info, 5);
        
        assert!(!modal.apply_to_all);
        modal.handle_key(KeyEvent::from(KeyCode::Char('a')));
        assert!(modal.apply_to_all);
    }
}
