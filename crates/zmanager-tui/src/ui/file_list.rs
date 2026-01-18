//! File list widget for displaying directory entries.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};
use zmanager_core::{EntryKind, EntryMeta};

use super::styles::Styles;

/// File list widget for displaying a list of entries.
pub struct FileList<'a> {
    entries: &'a [EntryMeta],
    selected_indices: &'a [usize],
    is_active: bool,
    title: Option<&'a str>,
}

impl<'a> FileList<'a> {
    /// Create a new file list widget.
    pub fn new(entries: &'a [EntryMeta], selected_indices: &'a [usize], is_active: bool) -> Self {
        Self {
            entries,
            selected_indices,
            is_active,
            title: None,
        }
    }

    /// Set the title for the file list.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Format file size for display.
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if size >= TB {
            format!("{:.1}T", size as f64 / TB as f64)
        } else if size >= GB {
            format!("{:.1}G", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.0}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }

    /// Get the icon for an entry kind.
    fn icon(kind: EntryKind) -> &'static str {
        match kind {
            EntryKind::Directory => "📁",
            EntryKind::File => "📄",
            EntryKind::Symlink => "🔗",
            EntryKind::Junction => "⛓️",
        }
    }

    /// Get style for an entry.
    fn entry_style(entry: &EntryMeta, is_selected: bool) -> ratatui::style::Style {
        let base = if entry.attributes.hidden {
            Styles::hidden()
        } else {
            match entry.kind {
                EntryKind::Directory => Styles::directory(),
                EntryKind::Symlink | EntryKind::Junction => Styles::normal(),
                EntryKind::File => {
                    if let Some(ext) = entry.extension.as_deref() {
                        Styles::for_extension(ext)
                    } else {
                        Styles::normal()
                    }
                }
            }
        };

        if is_selected {
            base.patch(Styles::selected())
        } else {
            base
        }
    }

    /// Render an entry as a list item.
    fn render_entry(&self, entry: &EntryMeta, is_selected: bool, width: u16) -> ListItem<'a> {
        let icon = Self::icon(entry.kind);
        let name = &entry.name;
        let style = Self::entry_style(entry, is_selected);

        // Calculate available width for name
        // Format: "📁 name          12.3M"
        let size_str = match entry.kind {
            EntryKind::Directory => "<DIR>".to_string(),
            _ => Self::format_size(entry.size),
        };

        let icon_width = 3; // icon + space
        let size_width = 8;
        let name_width = width.saturating_sub(icon_width + size_width) as usize;

        // Truncate or pad name
        let display_name = if name.len() > name_width {
            format!("{}…", &name[..name_width.saturating_sub(1)])
        } else {
            format!("{:width$}", name, width = name_width)
        };

        let line = Line::from(vec![
            Span::raw(format!("{} ", icon)),
            Span::styled(display_name, style),
            Span::styled(format!("{:>7}", size_str), Styles::size()),
        ]);

        ListItem::new(line)
    }
}

impl StatefulWidget for FileList<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_style = if self.is_active {
            Styles::active_border()
        } else {
            Styles::inactive_border()
        };

        let mut block = Block::default().borders(Borders::ALL).border_style(border_style);

        if let Some(title) = self.title {
            block = block.title(title);
        }

        let inner = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_selected = self.selected_indices.contains(&i);
                self.render_entry(entry, is_selected, inner.width)
            })
            .collect();

        let list = List::new(items).highlight_style(Styles::cursor());

        StatefulWidget::render(list, inner, buf, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(FileList::format_size(512), "512B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(FileList::format_size(2048), "2K");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(FileList::format_size(5 * 1024 * 1024), "5.0M");
    }

    #[test]
    fn format_size_gigabytes() {
        assert_eq!(FileList::format_size(2 * 1024 * 1024 * 1024), "2.0G");
    }

    #[test]
    fn icon_for_directory() {
        assert_eq!(FileList::icon(EntryKind::Directory), "📁");
    }

    #[test]
    fn icon_for_file() {
        assert_eq!(FileList::icon(EntryKind::File), "📄");
    }
}
