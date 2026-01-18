//! Quick Access sidebar with favorites and drives.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, StatefulWidget, Widget},
};
use zmanager_core::{DriveInfo, Favorite};

use super::styles::Styles;

/// Which section of the sidebar is focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarSection {
    #[default]
    Favorites,
    Drives,
}

impl SidebarSection {
    /// Toggle between sections.
    pub fn toggle(&self) -> Self {
        match self {
            Self::Favorites => Self::Drives,
            Self::Drives => Self::Favorites,
        }
    }
}

/// Quick Access sidebar widget.
pub struct Sidebar<'a> {
    favorites: &'a [Favorite],
    drives: &'a [DriveInfo],
    active_section: SidebarSection,
}

impl<'a> Sidebar<'a> {
    /// Create a new sidebar.
    pub fn new(favorites: &'a [Favorite], drives: &'a [DriveInfo], active_section: SidebarSection) -> Self {
        Self {
            favorites,
            drives,
            active_section,
        }
    }
}

impl StatefulWidget for Sidebar<'_> {
    type State = SidebarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear the area first
        Clear.render(area, buf);

        // Split into favorites section and drives section
        let chunks = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

        // Render favorites section
        let favorites_block = Block::default()
            .title(" ★ Favorites ")
            .borders(Borders::ALL)
            .border_style(if self.active_section == SidebarSection::Favorites {
                Styles::selected()
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let favorites_inner = favorites_block.inner(chunks[0]);
        favorites_block.render(chunks[0], buf);

        if self.favorites.is_empty() {
            let empty_msg = Line::from(Span::styled(
                "No favorites",
                Style::default().fg(Color::DarkGray),
            ));
            buf.set_line(
                favorites_inner.x + 1,
                favorites_inner.y,
                &empty_msg,
                favorites_inner.width.saturating_sub(2),
            );
        } else {
            let items: Vec<ListItem> = self
                .favorites
                .iter()
                .enumerate()
                .map(|(i, fav)| {
                    let number = if i < 9 {
                        format!("{} ", i + 1)
                    } else {
                        "  ".to_string()
                    };
                    let icon = if fav.is_broken() { "⚠" } else { "📁" };
                    let style = if fav.is_broken() {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(number, Style::default().fg(Color::DarkGray)),
                        Span::raw(icon),
                        Span::raw(" "),
                        Span::styled(&fav.name, style),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .highlight_style(Styles::selected())
                .highlight_symbol("▶ ");

            let mut fav_state = state.favorites_state.clone();
            StatefulWidget::render(list, favorites_inner, buf, &mut fav_state);
        }

        // Render drives section
        let drives_block = Block::default()
            .title(" 💾 Drives ")
            .borders(Borders::ALL)
            .border_style(if self.active_section == SidebarSection::Drives {
                Styles::selected()
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let drives_inner = drives_block.inner(chunks[1]);
        drives_block.render(chunks[1], buf);

        if self.drives.is_empty() {
            let empty_msg = Line::from(Span::styled(
                "No drives",
                Style::default().fg(Color::DarkGray),
            ));
            buf.set_line(
                drives_inner.x + 1,
                drives_inner.y,
                &empty_msg,
                drives_inner.width.saturating_sub(2),
            );
        } else {
            let items: Vec<ListItem> = self
                .drives
                .iter()
                .map(|drive| {
                    let icon = drive_icon(drive);
                    let label = drive.display_name();
                    let free = drive
                        .free_bytes
                        .map(|b| format!(" ({} free)", format_size(b)))
                        .unwrap_or_default();

                    let style = if !drive.is_ready {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    ListItem::new(Line::from(vec![
                        Span::raw(icon),
                        Span::raw(" "),
                        Span::styled(label, style),
                        Span::styled(free, Style::default().fg(Color::DarkGray)),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .highlight_style(Styles::selected())
                .highlight_symbol("▶ ");

            let mut drives_state = state.drives_state.clone();
            StatefulWidget::render(list, drives_inner, buf, &mut drives_state);
        }
    }
}

/// State for the sidebar widget.
#[derive(Debug, Clone, Default)]
pub struct SidebarState {
    /// Currently focused section.
    pub section: SidebarSection,
    /// Favorites list state.
    pub favorites_state: ListState,
    /// Drives list state.
    pub drives_state: ListState,
}

impl SidebarState {
    /// Create new sidebar state.
    pub fn new() -> Self {
        let mut state = Self::default();
        state.favorites_state.select(Some(0));
        state.drives_state.select(Some(0));
        state
    }

    /// Move selection up in the current section.
    pub fn up(&mut self, favorites_count: usize, drives_count: usize) {
        match self.section {
            SidebarSection::Favorites => {
                if favorites_count == 0 {
                    return;
                }
                let current = self.favorites_state.selected().unwrap_or(0);
                if current > 0 {
                    self.favorites_state.select(Some(current - 1));
                }
            }
            SidebarSection::Drives => {
                if drives_count == 0 {
                    return;
                }
                let current = self.drives_state.selected().unwrap_or(0);
                if current > 0 {
                    self.drives_state.select(Some(current - 1));
                }
            }
        }
    }

    /// Move selection down in the current section.
    pub fn down(&mut self, favorites_count: usize, drives_count: usize) {
        match self.section {
            SidebarSection::Favorites => {
                if favorites_count == 0 {
                    return;
                }
                let current = self.favorites_state.selected().unwrap_or(0);
                if current < favorites_count.saturating_sub(1) {
                    self.favorites_state.select(Some(current + 1));
                }
            }
            SidebarSection::Drives => {
                if drives_count == 0 {
                    return;
                }
                let current = self.drives_state.selected().unwrap_or(0);
                if current < drives_count.saturating_sub(1) {
                    self.drives_state.select(Some(current + 1));
                }
            }
        }
    }

    /// Toggle between sections.
    pub fn toggle_section(&mut self) {
        self.section = self.section.toggle();
    }

    /// Get the selected favorite index.
    pub fn selected_favorite(&self) -> Option<usize> {
        self.favorites_state.selected()
    }

    /// Get the selected drive index.
    pub fn selected_drive(&self) -> Option<usize> {
        self.drives_state.selected()
    }

    /// Select a favorite by number (1-9).
    pub fn select_by_number(&mut self, num: usize, favorites_count: usize) {
        if num > 0 && num <= favorites_count && num <= 9 {
            self.section = SidebarSection::Favorites;
            self.favorites_state.select(Some(num - 1));
        }
    }
}

/// Get icon for drive type.
fn drive_icon(drive: &DriveInfo) -> &'static str {
    use zmanager_core::DriveType;
    match drive.drive_type {
        DriveType::Fixed => "💿",
        DriveType::Removable => "💾",
        DriveType::Network => "🌐",
        DriveType::CdRom => "📀",
        DriveType::RamDisk => "🔧",
        _ => "💿",
    }
}

/// Format size for display.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_state_navigation() {
        let mut state = SidebarState::new();
        assert_eq!(state.section, SidebarSection::Favorites);

        state.toggle_section();
        assert_eq!(state.section, SidebarSection::Drives);

        state.toggle_section();
        assert_eq!(state.section, SidebarSection::Favorites);
    }

    #[test]
    fn sidebar_up_down() {
        let mut state = SidebarState::new();
        
        // Start at 0, go down
        state.down(3, 2);
        assert_eq!(state.selected_favorite(), Some(1));

        state.down(3, 2);
        assert_eq!(state.selected_favorite(), Some(2));

        // At end, stays at end
        state.down(3, 2);
        assert_eq!(state.selected_favorite(), Some(2));

        // Go back up
        state.up(3, 2);
        assert_eq!(state.selected_favorite(), Some(1));
    }

    #[test]
    fn sidebar_quick_jump() {
        let mut state = SidebarState::new();
        
        state.select_by_number(3, 5);
        assert_eq!(state.selected_favorite(), Some(2)); // 0-indexed

        // Invalid numbers don't change selection
        state.select_by_number(10, 5);
        assert_eq!(state.selected_favorite(), Some(2));
    }

    #[test]
    fn format_size_display() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024 * 100), "100.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 50), "50.0 GB");
    }
}
