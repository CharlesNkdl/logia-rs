use crate::app::{App, AppMode, LoadingState};
use crate::log::log_source::FsEntry;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, List, ListItem};

const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render_sidebar(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    // Show scanning spinner instead of list
    if app.loading == LoadingState::Scanning {
        let spinner = SPINNER_FRAMES[app.loading_frame as usize % SPINNER_FRAMES.len()];
        let block = Block::bordered()
            .title(" Log Explorer ")
            .border_type(BorderType::Rounded)
            .fg(theme.border_focused)
            .bg(ratatui::prelude::Color::Reset);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("\n  {} Scanning...", spinner))
                .block(block),
            area,
        );
        return;
    }

    // Filesystem navigation mode: show FsEntry list when current_dir is set
    if app.mode != AppMode::ServerSelect {
        if let Some(ref dir) = app.current_dir.clone() {
            let title = format!(" {} ", dir.to_string_lossy());

            if app.fs_entries.is_empty() {
                frame.render_widget(
                    ratatui::widgets::Paragraph::new("\n  No log files found within 3 levels")
                        .block(
                            Block::bordered()
                                .title(title)
                                .border_type(BorderType::Rounded)
                                .fg(theme.border_focused)
                                .bg(ratatui::prelude::Color::Reset),
                        ),
                    area,
                );
                return;
            }

            let items: Vec<ListItem> = app
                .fs_entries
                .iter()
                .map(|entry| match entry {
                    FsEntry::Directory(path) => {
                        let name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        ListItem::new(format!("📁 {}", name))
                            .style(Style::default().fg(theme.item_local))
                    }
                    FsEntry::LogFile(source) => {
                        ListItem::new(format!("📄 {}", source.name))
                            .style(Style::default().fg(theme.item_normal))
                    }
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::bordered()
                        .title(title)
                        .border_type(BorderType::Rounded)
                        .fg(theme.border_focused)
                        .bg(ratatui::prelude::Color::Reset),
                )
                .highlight_style(Style::default().fg(theme.highlight_fg).bg(theme.highlight_bg))
                .highlight_symbol("▶ ");
            frame.render_stateful_widget(list, area, &mut app.list_state);
            return;
        }
    }

    let title: &str;
    let items: Vec<ListItem> = match app.mode {
        AppMode::ServerSelect => {
            title = " Server Selection ";
            let mut list = vec![
                ListItem::new("Local (This Machine)").style(Style::default().fg(theme.item_local)),
            ];
            for server in &app.config.servers {
                list.push(
                    ListItem::new(format!("SSH: {} ({})", server.name, server.host))
                        .style(Style::default().fg(theme.item_normal)),
                );
            }
            list
        }
        _ => {
            title = " Log Explorer ";
            app.filtered_sources
                .iter()
                .map(|s| ListItem::new(s.name.clone()).style(Style::default().fg(theme.item_normal)))
                .collect()
        }
    };

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(title)
                .border_type(BorderType::Rounded)
                .fg(theme.border_focused)
                .bg(ratatui::prelude::Color::Reset),
        )
        .highlight_style(Style::default().fg(theme.highlight_fg).bg(theme.highlight_bg))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}
