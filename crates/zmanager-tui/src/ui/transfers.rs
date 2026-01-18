//! Transfers view widget showing active jobs and progress.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use zmanager_core::{JobInfo, JobState};

use super::styles::Styles;

/// Transfers view panel showing active/completed jobs.
pub struct TransfersView<'a> {
    jobs: &'a [JobInfo],
    is_active: bool,
}

impl<'a> TransfersView<'a> {
    /// Create a new transfers view.
    pub fn new(jobs: &'a [JobInfo], is_active: bool) -> Self {
        Self { jobs, is_active }
    }
}

impl StatefulWidget for TransfersView<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear area
        Clear.render(area, buf);

        // Create bordered block
        let border_style = if self.is_active {
            Styles::active_border()
        } else {
            Styles::inactive_border()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Transfers (t to close) ");

        let inner = block.inner(area);
        block.render(area, buf);

        if self.jobs.is_empty() {
            // Show empty message
            Paragraph::new("No active transfers")
                .alignment(Alignment::Center)
                .style(Style::default().add_modifier(Modifier::DIM))
                .render(inner, buf);
            return;
        }

        // Create list items for each job
        let items: Vec<ListItem> = self
            .jobs
            .iter()
            .map(|job| create_job_item(job))
            .collect();

        let list = List::new(items)
            .highlight_style(Styles::selected());

        StatefulWidget::render(list, inner, buf, state);
    }
}

fn create_job_item(job: &JobInfo) -> ListItem<'static> {
    // Format: [State] Description | Progress Bar | Speed | ETA
    let state_span = match job.state {
        JobState::Pending => Span::styled("⏳", Style::default().fg(Color::Yellow)),
        JobState::Running => Span::styled("▶", Style::default().fg(Color::Green)),
        JobState::Paused => Span::styled("⏸", Style::default().fg(Color::Blue)),
        JobState::Completed => Span::styled("✓", Style::default().fg(Color::Green)),
        JobState::Failed => Span::styled("✗", Style::default().fg(Color::Red)),
        JobState::Cancelled => Span::styled("⊘", Style::default().fg(Color::DarkGray)),
    };

    let desc = Span::raw(format!(" {} ", truncate_string(&job.description, 30)));

    let progress = format!("{}%", job.progress_percent);
    let progress_span = Span::styled(
        format!("{:>4}", progress),
        Style::default().fg(progress_color(job.progress_percent)),
    );

    let speed = job.speed_bytes_per_sec
        .map(format_speed)
        .unwrap_or_else(|| "---".to_string());
    let speed_span = Span::styled(format!(" {:>10}", speed), Style::default().fg(Color::Cyan));

    let eta = job.eta_secs
        .map(format_eta)
        .unwrap_or_else(|| "---".to_string());
    let eta_span = Span::styled(format!(" {:>8}", eta), Style::default().fg(Color::Magenta));

    ListItem::new(Line::from(vec![
        state_span,
        desc,
        progress_span,
        speed_span,
        eta_span,
    ]))
}

fn progress_color(percent: u8) -> Color {
    match percent {
        0..=25 => Color::Red,
        26..=50 => Color::Yellow,
        51..=75 => Color::LightYellow,
        76..=99 => Color::LightGreen,
        _ => Color::Green,
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes_per_sec >= GB {
        format!("{:.1} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

fn format_eta(secs: u64) -> String {
    if secs >= 3600 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {:02}m", hours, mins)
    } else if secs >= 60 {
        let mins = secs / 60;
        let s = secs % 60;
        format!("{}m {:02}s", mins, s)
    } else {
        format!("{}s", secs)
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}

/// A progress bar widget for a single job.
pub struct JobProgressBar {
    pub percent: u8,
    pub label: String,
}

impl Widget for JobProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let gauge = Gauge::default()
            .percent(self.percent as u16)
            .label(self.label)
            .gauge_style(Style::default().fg(progress_color(self.percent)));

        gauge.render(area, buf);
    }
}

/// Transfer status indicator for the status bar.
#[derive(Debug, Clone)]
pub struct TransferStatus {
    pub active_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

impl TransferStatus {
    pub fn new(active: usize, completed: usize, failed: usize) -> Self {
        Self {
            active_count: active,
            completed_count: completed,
            failed_count: failed,
        }
    }

    /// Format for status bar display.
    pub fn format(&self) -> String {
        if self.active_count == 0 && self.completed_count == 0 && self.failed_count == 0 {
            return String::new();
        }

        let mut parts = Vec::new();
        if self.active_count > 0 {
            parts.push(format!("{}▶", self.active_count));
        }
        if self.completed_count > 0 {
            parts.push(format!("{}✓", self.completed_count));
        }
        if self.failed_count > 0 {
            parts.push(format!("{}✗", self.failed_count));
        }

        format!(" [{}]", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_speed_bytes() {
        assert_eq!(format_speed(512), "512 B/s");
    }

    #[test]
    fn format_speed_kilobytes() {
        assert_eq!(format_speed(1024), "1.0 KB/s");
        assert_eq!(format_speed(2048), "2.0 KB/s");
    }

    #[test]
    fn format_speed_megabytes() {
        assert_eq!(format_speed(1024 * 1024), "1.0 MB/s");
        assert_eq!(format_speed(5 * 1024 * 1024), "5.0 MB/s");
    }

    #[test]
    fn format_speed_gigabytes() {
        assert_eq!(format_speed(1024 * 1024 * 1024), "1.0 GB/s");
    }

    #[test]
    fn format_eta_seconds() {
        assert_eq!(format_eta(30), "30s");
    }

    #[test]
    fn format_eta_minutes() {
        assert_eq!(format_eta(90), "1m 30s");
        assert_eq!(format_eta(120), "2m 00s");
    }

    #[test]
    fn format_eta_hours() {
        assert_eq!(format_eta(3661), "1h 01m");
    }

    #[test]
    fn transfer_status_format() {
        let status = TransferStatus::new(2, 3, 1);
        assert!(status.format().contains("2▶"));
        assert!(status.format().contains("3✓"));
        assert!(status.format().contains("1✗"));
    }

    #[test]
    fn transfer_status_empty() {
        let status = TransferStatus::new(0, 0, 0);
        assert_eq!(status.format(), "");
    }
}
