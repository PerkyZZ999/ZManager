//! Main layout structure for the TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Layout areas for the application.
#[derive(Debug, Clone, Copy)]
pub struct AppLayout {
    /// Left pane header area.
    pub left_header: Rect,
    /// Right pane header area.
    pub right_header: Rect,
    /// Left pane content area.
    pub left_content: Rect,
    /// Right pane content area.
    pub right_content: Rect,
    /// Status bar area (bottom).
    pub status: Rect,
}

impl AppLayout {
    /// Create layout from the terminal frame.
    pub fn new(frame: &Frame) -> Self {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Content + headers
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area());

        // Split content area into left and right panes
        let pane_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[0]);

        // Split each pane into header + content
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(pane_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(pane_chunks[1]);

        Self {
            left_header: left_chunks[0],
            right_header: right_chunks[0],
            left_content: left_chunks[1],
            right_content: right_chunks[1],
            status: main_chunks[1],
        }
    }

    /// Backward compatibility: get header area (left pane header).
    #[deprecated(note = "Use left_header and right_header instead")]
    pub fn header(&self) -> Rect {
        self.left_header
    }

    /// Backward compatibility: get content area.
    #[deprecated(note = "Use left_content and right_content instead")]
    pub fn content(&self) -> Rect {
        Rect {
            x: self.left_content.x,
            y: self.left_content.y,
            width: self.left_content.width + self.right_content.width,
            height: self.left_content.height,
        }
    }

    /// Get dual pane content areas.
    pub fn dual_panes(&self) -> (Rect, Rect) {
        (self.left_content, self.right_content)
    }

    /// Get single pane (full width) - combines both panes.
    pub fn single_pane(&self) -> Rect {
        Rect {
            x: self.left_content.x,
            y: self.left_content.y.saturating_sub(1), // Include header
            width: self.left_content.width + self.right_content.width,
            height: self.left_content.height + 1,
        }
    }
}

/// Pane identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    #[default]
    Left,
    Right,
}

impl Pane {
    /// Toggle to the other pane.
    pub fn toggle(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_toggle() {
        assert_eq!(Pane::Left.toggle(), Pane::Right);
        assert_eq!(Pane::Right.toggle(), Pane::Left);
    }

    #[test]
    fn pane_default_is_left() {
        assert_eq!(Pane::default(), Pane::Left);
    }
}
