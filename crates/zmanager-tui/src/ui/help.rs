//! Help screen widget showing keyboard shortcuts.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

/// Help screen widget.
pub struct HelpScreen;

impl HelpScreen {
    /// Create a new help screen.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HelpScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear and render centered modal
        let modal_width = 70.min(area.width.saturating_sub(4));
        let modal_height = 32.min(area.height.saturating_sub(4));
        
        let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;
        
        let modal_area = Rect {
            x: modal_x,
            y: modal_y,
            width: modal_width,
            height: modal_height,
        };

        Clear.render(modal_area, buf);

        let block = Block::default()
            .title(" Help - Keyboard Shortcuts ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(modal_area);
        block.render(modal_area, buf);

        // Build help content
        let sections = [
            ("Navigation", vec![
                ("↑/k, ↓/j", "Move cursor up/down"),
                ("←/h, →/l", "Parent directory / Enter"),
                ("Enter", "Enter directory / Open file"),
                ("Backspace", "Go to parent directory"),
                ("Tab", "Switch between panes"),
                ("g/Home", "Go to first item"),
                ("G/End", "Go to last item"),
                ("Ctrl+u/PgUp", "Page up"),
                ("Ctrl+d/PgDn", "Page down"),
                ("[/]", "History back/forward"),
            ]),
            ("Selection", vec![
                ("Space", "Toggle selection"),
                ("Ctrl+a", "Select all"),
                ("*", "Invert selection"),
                ("Esc", "Clear selection"),
            ]),
            ("File Operations", vec![
                ("Shift+C", "Copy to other pane"),
                ("Shift+M", "Move to other pane"),
                ("d/Del", "Delete selected"),
                ("r/F2", "Rename"),
                ("n", "New directory"),
                ("o", "Open with default app"),
            ]),
            ("Views & Panels", vec![
                ("t", "Toggle transfers view"),
                ("Ctrl+b", "Toggle sidebar"),
                (".", "Toggle hidden files"),
                ("s", "Sort menu"),
                ("i", "Properties"),
                ("?/F1", "This help screen"),
            ]),
            ("Transfers", vec![
                ("Shift+P", "Pause job"),
                ("Shift+R", "Resume job"),
                ("Shift+X", "Cancel job"),
            ]),
            ("Quick Access", vec![
                ("Ctrl+d", "Add to favorites"),
                ("1-9", "Quick jump to favorite"),
            ]),
            ("General", vec![
                ("q/Ctrl+c", "Quit"),
                ("F5/Ctrl+r", "Refresh"),
            ]),
        ];

        // Calculate column layout
        let content_width = inner.width as usize;
        let key_width = 14;

        let mut lines: Vec<Line> = Vec::new();
        
        for (section_name, shortcuts) in &sections {
            // Section header
            lines.push(Line::from(vec![
                Span::styled(
                    format!("─── {} ", section_name),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "─".repeat(content_width.saturating_sub(section_name.len() + 5)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Shortcuts
            for (key, desc) in shortcuts {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:width$}", key, width = key_width),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(*desc, Style::default()),
                ]));
            }
            
            lines.push(Line::from(""));
        }

        // Footer
        lines.push(Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false });

        paragraph.render(inner, buf);
    }
}

/// Handle key input for help screen.
/// Returns true if the help screen should be closed.
pub fn handle_help_key(_key: crossterm::event::KeyEvent) -> bool {
    // Any key closes the help screen
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_screen_closes_on_any_key() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(handle_help_key(key));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(handle_help_key(key));

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(handle_help_key(key));
    }
}
