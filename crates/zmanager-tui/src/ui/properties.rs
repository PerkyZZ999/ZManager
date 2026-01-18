//! Properties panel widget showing file/folder details.

use std::path::Path;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use zmanager_core::Properties;

/// Properties panel widget.
pub struct PropertiesPanel<'a> {
    properties: &'a Properties,
}

impl<'a> PropertiesPanel<'a> {
    /// Create a new properties panel.
    pub fn new(properties: &'a Properties) -> Self {
        Self { properties }
    }
}

impl Widget for PropertiesPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate modal size
        let modal_width = 60.min(area.width.saturating_sub(4));
        let modal_height = 20.min(area.height.saturating_sub(4));

        let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: modal_x,
            y: modal_y,
            width: modal_width,
            height: modal_height,
        };

        Clear.render(modal_area, buf);

        let title = format!(" {} ", self.properties.name);
        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(modal_area);
        block.render(modal_area, buf);

        // Build properties content
        let label_style = Style::default().fg(Color::DarkGray);
        let value_style = Style::default();
        let highlight_style = Style::default().fg(Color::Yellow);

        let mut lines: Vec<Line> = Vec::new();

        // Type
        let is_dir = self.properties.kind.is_directory();
        let type_icon = if is_dir { "📁" } else { "📄" };
        let type_name = if is_dir { "Folder" } else { "File" };
        lines.push(Line::from(vec![
            Span::styled("Type:         ", label_style),
            Span::raw(type_icon),
            Span::raw(" "),
            Span::styled(type_name, value_style),
        ]));

        // Location
        let parent = Path::new(&self.properties.path)
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled("Location:     ", label_style),
            Span::styled(truncate_path(&parent, inner.width as usize - 15), value_style),
        ]));

        lines.push(Line::from(""));

        // Size
        let size_display = self.properties.size_display();
        let size_bytes = self.properties.size.unwrap_or(0);
        lines.push(Line::from(vec![
            Span::styled("Size:         ", label_style),
            Span::styled(&size_display, highlight_style),
            Span::styled(
                format!(" ({} bytes)", format_bytes_with_commas(size_bytes)),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Folder contents summary
        if let Some(summary) = self.properties.contents_summary() {
            lines.push(Line::from(vec![
                Span::styled("Contains:     ", label_style),
                Span::styled(summary, value_style),
            ]));
        }

        lines.push(Line::from(""));

        // Dates
        if let Some(created) = self.properties.created_display() {
            lines.push(Line::from(vec![
                Span::styled("Created:      ", label_style),
                Span::styled(created, value_style),
            ]));
        }

        if let Some(modified) = self.properties.modified_display() {
            lines.push(Line::from(vec![
                Span::styled("Modified:     ", label_style),
                Span::styled(modified, value_style),
            ]));
        }

        lines.push(Line::from(""));

        // Attributes
        let mut attrs = Vec::new();
        if self.properties.readonly {
            attrs.push("Read-only");
        }
        if self.properties.hidden {
            attrs.push("Hidden");
        }
        if self.properties.system {
            attrs.push("System");
        }
        if self.properties.link_target.is_some() {
            attrs.push("Symlink");
        }

        if !attrs.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Attributes:   ", label_style),
                Span::styled(attrs.join(", "), value_style),
            ]));
        }

        // MIME type
        if let Some(ref mime) = self.properties.mime_type {
            lines.push(Line::from(vec![
                Span::styled("MIME Type:    ", label_style),
                Span::styled(mime, value_style),
            ]));
        }

        // Symlink target
        if let Some(ref target) = self.properties.link_target {
            lines.push(Line::from(vec![
                Span::styled("Target:       ", label_style),
                Span::styled(target.display().to_string(), value_style),
            ]));
        }

        lines.push(Line::from(""));
        
        // Footer
        lines.push(Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));

        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);
    }
}

/// Handle key input for properties panel.
/// Returns true if the panel should be closed.
pub fn handle_properties_key(_key: crossterm::event::KeyEvent) -> bool {
    // Any key closes the panel
    true
}

/// Format bytes with thousand separators.
fn format_bytes_with_commas(bytes: u64) -> String {
    let s = bytes.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Truncate a path to fit within a width.
fn truncate_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        path.to_string()
    } else if max_width <= 5 {
        "...".to_string()
    } else {
        format!("...{}", &path[path.len() - (max_width - 3)..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_commas() {
        assert_eq!(format_bytes_with_commas(100), "100");
        assert_eq!(format_bytes_with_commas(1000), "1,000");
        assert_eq!(format_bytes_with_commas(1000000), "1,000,000");
        assert_eq!(format_bytes_with_commas(1234567890), "1,234,567,890");
    }

    #[test]
    fn truncate_long_path() {
        assert_eq!(truncate_path("short", 20), "short");
        assert_eq!(truncate_path("C:\\very\\long\\path\\to\\file", 15), "...path\\to\\file");
    }

    #[test]
    fn properties_panel_closes_on_any_key() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(handle_properties_key(key));
    }
}
