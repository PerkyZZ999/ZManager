//! Status bar widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::styles::Styles;

/// Status bar showing current state information.
pub struct StatusBar<'a> {
    /// Number of entries in current directory.
    entry_count: usize,
    /// Number of selected entries.
    selected_count: usize,
    /// Total size of selected entries.
    selected_size: u64,
    /// Optional status message.
    message: Option<&'a str>,
    /// Whether a job is in progress.
    job_in_progress: bool,
}

impl<'a> StatusBar<'a> {
    /// Create a new status bar.
    pub fn new(entry_count: usize, selected_count: usize, selected_size: u64) -> Self {
        Self {
            entry_count,
            selected_count,
            selected_size,
            message: None,
            job_in_progress: false,
        }
    }

    /// Set a status message.
    pub fn message(mut self, msg: &'a str) -> Self {
        self.message = Some(msg);
        self
    }

    /// Set job in progress flag.
    pub fn job_in_progress(mut self, in_progress: bool) -> Self {
        self.job_in_progress = in_progress;
        self
    }

    /// Format file size for display.
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::new();

        // Left side: entry count and selection info
        spans.push(Span::styled(
            format!(" {} items", self.entry_count),
            Styles::status_bar(),
        ));

        if self.selected_count > 0 {
            spans.push(Span::styled(
                format!(
                    " | {} selected ({})",
                    self.selected_count,
                    Self::format_size(self.selected_size)
                ),
                Styles::status_bar(),
            ));
        }

        // Job indicator
        if self.job_in_progress {
            spans.push(Span::styled(" | ⏳ Working...", Styles::warning()));
        }

        // Message (if any)
        if let Some(msg) = self.message {
            spans.push(Span::styled(format!(" | {}", msg), Styles::status_bar()));
        }

        // Fill remaining space
        let content_len: usize = spans.iter().map(|s| s.content.len()).sum();
        let padding = area.width.saturating_sub(content_len as u16) as usize;
        spans.push(Span::styled(" ".repeat(padding), Styles::status_bar()));

        let line = Line::from(spans);
        Paragraph::new(line).style(Styles::status_bar()).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(StatusBar::format_size(100), "100 B");
    }

    #[test]
    fn format_size_kb() {
        assert_eq!(StatusBar::format_size(2048), "2.0 KB");
    }

    #[test]
    fn format_size_mb() {
        assert_eq!(StatusBar::format_size(5 * 1024 * 1024), "5.00 MB");
    }

    #[test]
    fn format_size_gb() {
        assert_eq!(StatusBar::format_size(3 * 1024 * 1024 * 1024), "3.00 GB");
    }

    #[test]
    fn status_bar_with_selection() {
        let bar = StatusBar::new(100, 5, 1024 * 1024);
        // Just ensure it doesn't panic
        assert_eq!(bar.selected_count, 5);
    }
}
