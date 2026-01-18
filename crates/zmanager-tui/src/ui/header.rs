//! Header widget for path display.

use std::path::Path;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::styles::Styles;

/// Header widget showing the current path.
pub struct Header<'a> {
    path: &'a Path,
    is_active: bool,
}

impl<'a> Header<'a> {
    /// Create a new header widget.
    pub fn new(path: &'a Path, is_active: bool) -> Self {
        Self { path, is_active }
    }

    /// Build breadcrumb spans from path.
    fn breadcrumbs(&self) -> Line<'a> {
        let mut spans = Vec::new();
        let style = if self.is_active {
            Styles::header()
        } else {
            Styles::normal()
        };

        // Handle Windows drive letter
        if let Some(prefix) = self.path.components().next() {
            spans.push(Span::styled(prefix.as_os_str().to_string_lossy().to_string(), style));
        }

        for component in self.path.components().skip(1) {
            let os_str = component.as_os_str();
            if !os_str.is_empty() {
                spans.push(Span::styled(" › ", Styles::normal()));
                spans.push(Span::styled(os_str.to_string_lossy().to_string(), style));
            }
        }

        if spans.is_empty() {
            spans.push(Span::styled(
                self.path.display().to_string(),
                style,
            ));
        }

        Line::from(spans)
    }
}

impl Widget for Header<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.breadcrumbs();
        Paragraph::new(line).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_creates_breadcrumbs() {
        let path = Path::new("C:\\Users\\Test\\Documents");
        let header = Header::new(path, true);
        let line = header.breadcrumbs();
        
        // Should have spans for each component
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn header_handles_root() {
        let path = Path::new("C:\\");
        let header = Header::new(path, false);
        let line = header.breadcrumbs();
        
        assert!(!line.spans.is_empty());
    }
}
