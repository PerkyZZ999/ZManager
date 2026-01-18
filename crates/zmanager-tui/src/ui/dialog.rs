//! Dialog and prompt widgets for user input.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use super::styles::Styles;

/// Dialog type for different operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    /// Confirmation dialog (Yes/No).
    Confirm {
        title: String,
        message: String,
    },
    /// Text input dialog.
    Input {
        title: String,
        prompt: String,
        value: String,
        cursor_pos: usize,
    },
    /// Information/error message.
    Message {
        title: String,
        message: String,
        is_error: bool,
    },
    /// Sort selection menu.
    SortMenu {
        current: SortField,
    },
}

/// Sort field options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortField {
    #[default]
    Name,
    Size,
    Modified,
    Extension,
    Kind,
}

impl SortField {
    /// Get all sort fields.
    pub fn all() -> &'static [SortField] {
        &[
            SortField::Name,
            SortField::Size,
            SortField::Modified,
            SortField::Extension,
            SortField::Kind,
        ]
    }

    /// Get the label for this field.
    pub fn label(&self) -> &'static str {
        match self {
            SortField::Name => "Name",
            SortField::Size => "Size",
            SortField::Modified => "Modified",
            SortField::Extension => "Extension",
            SortField::Kind => "Kind",
        }
    }

    /// Get the hotkey for this field.
    pub fn hotkey(&self) -> char {
        match self {
            SortField::Name => 'n',
            SortField::Size => 's',
            SortField::Modified => 'm',
            SortField::Extension => 'e',
            SortField::Kind => 'k',
        }
    }
}

/// Result of dialog interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResult {
    /// Dialog is still open.
    Open,
    /// User confirmed (Yes, Enter).
    Confirmed(String),
    /// User cancelled (No, Escape).
    Cancelled,
    /// Sort field selected.
    SortSelected(SortField),
}

/// Active dialog state.
#[derive(Debug, Clone)]
pub struct Dialog {
    pub kind: DialogKind,
}

impl Dialog {
    /// Create a confirmation dialog.
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: DialogKind::Confirm {
                title: title.into(),
                message: message.into(),
            },
        }
    }

    /// Create an input dialog.
    pub fn input(title: impl Into<String>, prompt: impl Into<String>, initial: impl Into<String>) -> Self {
        let value = initial.into();
        let cursor_pos = value.len();
        Self {
            kind: DialogKind::Input {
                title: title.into(),
                prompt: prompt.into(),
                value,
                cursor_pos,
            },
        }
    }

    /// Create a message dialog.
    pub fn message(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: DialogKind::Message {
                title: title.into(),
                message: message.into(),
                is_error: false,
            },
        }
    }

    /// Create an error dialog.
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: DialogKind::Message {
                title: title.into(),
                message: message.into(),
                is_error: true,
            },
        }
    }

    /// Create a sort menu.
    pub fn sort_menu(current: SortField) -> Self {
        Self {
            kind: DialogKind::SortMenu { current },
        }
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) -> DialogResult {
        match &mut self.kind {
            DialogKind::Confirm { .. } => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    DialogResult::Confirmed(String::new())
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => DialogResult::Cancelled,
                _ => DialogResult::Open,
            },
            DialogKind::Input {
                value, cursor_pos, ..
            } => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Enter) => DialogResult::Confirmed(value.clone()),
                (KeyModifiers::NONE, KeyCode::Esc) => DialogResult::Cancelled,
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    if *cursor_pos > 0 {
                        value.remove(*cursor_pos - 1);
                        *cursor_pos -= 1;
                    }
                    DialogResult::Open
                }
                (KeyModifiers::NONE, KeyCode::Delete) => {
                    if *cursor_pos < value.len() {
                        value.remove(*cursor_pos);
                    }
                    DialogResult::Open
                }
                (KeyModifiers::NONE, KeyCode::Left) => {
                    *cursor_pos = cursor_pos.saturating_sub(1);
                    DialogResult::Open
                }
                (KeyModifiers::NONE, KeyCode::Right) => {
                    *cursor_pos = (*cursor_pos + 1).min(value.len());
                    DialogResult::Open
                }
                (KeyModifiers::NONE, KeyCode::Home) => {
                    *cursor_pos = 0;
                    DialogResult::Open
                }
                (KeyModifiers::NONE, KeyCode::End) => {
                    *cursor_pos = value.len();
                    DialogResult::Open
                }
                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    value.insert(*cursor_pos, c);
                    *cursor_pos += 1;
                    DialogResult::Open
                }
                _ => DialogResult::Open,
            },
            DialogKind::Message { .. } => match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => DialogResult::Cancelled,
                _ => DialogResult::Open,
            },
            DialogKind::SortMenu { current } => match key.code {
                KeyCode::Esc => DialogResult::Cancelled,
                KeyCode::Char('n') => DialogResult::SortSelected(SortField::Name),
                KeyCode::Char('s') => DialogResult::SortSelected(SortField::Size),
                KeyCode::Char('m') => DialogResult::SortSelected(SortField::Modified),
                KeyCode::Char('e') => DialogResult::SortSelected(SortField::Extension),
                KeyCode::Char('k') => DialogResult::SortSelected(SortField::Kind),
                KeyCode::Enter => DialogResult::SortSelected(*current),
                _ => DialogResult::Open,
            },
        }
    }

    /// Render the dialog.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Calculate dialog size and position (centered)
        let width = area.width.clamp(30, 60);
        let height = match &self.kind {
            DialogKind::Confirm { .. } => 5,
            DialogKind::Input { .. } => 5,
            DialogKind::Message { .. } => 5,
            DialogKind::SortMenu { .. } => 9,
        };

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        // Clear the area behind the dialog
        Clear.render(dialog_area, buf);

        match &self.kind {
            DialogKind::Confirm { title, message } => {
                self.render_confirm(dialog_area, buf, title, message);
            }
            DialogKind::Input {
                title,
                prompt,
                value,
                cursor_pos,
            } => {
                self.render_input(dialog_area, buf, title, prompt, value, *cursor_pos);
            }
            DialogKind::Message {
                title,
                message,
                is_error,
            } => {
                self.render_message(dialog_area, buf, title, message, *is_error);
            }
            DialogKind::SortMenu { current } => {
                self.render_sort_menu(dialog_area, buf, *current);
            }
        }
    }

    fn render_confirm(&self, area: Rect, buf: &mut Buffer, title: &str, message: &str) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Styles::active_border())
            .title(format!(" {} ", title));

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        // Message
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .render(chunks[0], buf);

        // Options
        let options = Line::from(vec![
            Span::styled("[Y]es", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[N]o", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        Paragraph::new(options)
            .alignment(Alignment::Center)
            .render(chunks[1], buf);
    }

    fn render_input(
        &self,
        area: Rect,
        buf: &mut Buffer,
        title: &str,
        prompt: &str,
        value: &str,
        cursor_pos: usize,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Styles::active_border())
            .title(format!(" {} ", title));

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        // Prompt
        Paragraph::new(prompt).render(chunks[0], buf);

        // Input field with cursor
        let input_area = chunks[1];
        let display_value = format!("{}_", value);
        
        // Show cursor by underlining the character at cursor position
        let mut spans = Vec::new();
        for (i, c) in display_value.chars().enumerate() {
            let style = if i == cursor_pos {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            spans.push(Span::styled(c.to_string(), style));
        }
        
        Paragraph::new(Line::from(spans)).render(input_area, buf);
    }

    fn render_message(&self, area: Rect, buf: &mut Buffer, title: &str, message: &str, is_error: bool) {
        let border_style = if is_error {
            Styles::error()
        } else {
            Styles::active_border()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(" {} ", title));

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        // Message
        let msg_style = if is_error { Styles::error() } else { Styles::normal() };
        Paragraph::new(Span::styled(message, msg_style))
            .alignment(Alignment::Center)
            .render(chunks[0], buf);

        // Dismiss hint
        Paragraph::new("Press Enter or Esc to close")
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM))
            .render(chunks[1], buf);
    }

    fn render_sort_menu(&self, area: Rect, buf: &mut Buffer, current: SortField) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Styles::active_border())
            .title(" Sort by ");

        let inner = block.inner(area);
        block.render(area, buf);

        for (i, field) in SortField::all().iter().enumerate() {
            if i >= inner.height as usize {
                break;
            }

            let is_current = *field == current;
            let marker = if is_current { "► " } else { "  " };
            let style = if is_current {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw(marker),
                Span::styled(format!("[{}] ", field.hotkey()), Styles::header()),
                Span::styled(field.label(), style),
            ]);

            let y = inner.y + i as u16;
            Paragraph::new(line).render(Rect::new(inner.x, y, inner.width, 1), buf);
        }

        // Hint at bottom
        let hint_y = inner.y + inner.height.saturating_sub(1);
        Paragraph::new("Press key or Esc to cancel")
            .style(Style::default().add_modifier(Modifier::DIM))
            .render(Rect::new(inner.x, hint_y, inner.width, 1), buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_dialog_yes() {
        let mut dialog = Dialog::confirm("Delete", "Delete file?");
        let result = dialog.handle_key(KeyEvent::from(KeyCode::Char('y')));
        assert_eq!(result, DialogResult::Confirmed(String::new()));
    }

    #[test]
    fn confirm_dialog_no() {
        let mut dialog = Dialog::confirm("Delete", "Delete file?");
        let result = dialog.handle_key(KeyEvent::from(KeyCode::Char('n')));
        assert_eq!(result, DialogResult::Cancelled);
    }

    #[test]
    fn input_dialog_typing() {
        let mut dialog = Dialog::input("Rename", "New name:", "test");
        
        // Type a character
        dialog.handle_key(KeyEvent::from(KeyCode::Char('!')));
        
        if let DialogKind::Input { value, .. } = &dialog.kind {
            assert_eq!(value, "test!");
        } else {
            panic!("Expected Input dialog");
        }
    }

    #[test]
    fn input_dialog_backspace() {
        let mut dialog = Dialog::input("Rename", "New name:", "test");
        
        dialog.handle_key(KeyEvent::from(KeyCode::Backspace));
        
        if let DialogKind::Input { value, .. } = &dialog.kind {
            assert_eq!(value, "tes");
        } else {
            panic!("Expected Input dialog");
        }
    }

    #[test]
    fn sort_menu_selection() {
        let mut dialog = Dialog::sort_menu(SortField::Name);
        let result = dialog.handle_key(KeyEvent::from(KeyCode::Char('s')));
        assert_eq!(result, DialogResult::SortSelected(SortField::Size));
    }
}
