use crate::app::{App, AppMode, LoadingState};
use crate::log::parsers::laravel::parse_laravel_line;
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Wrap},
};

const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render_main(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    if app.loading == LoadingState::Connecting {
        let spinner = SPINNER_FRAMES[app.loading_frame as usize % SPINNER_FRAMES.len()];
        frame.render_widget(
            Paragraph::new(format!("\n\n  {} Connecting...", spinner))
                .block(
                    Block::bordered()
                        .title(" Status ")
                        .border_type(BorderType::Rounded)
                        .fg(theme.border_focused)
                        .bg(theme.content_bg),
                )
                .alignment(Alignment::Center),
            area,
        );
        return;
    }
    if app.loading == LoadingState::Idle {
        if let Some(msg) = &app.status_message {
            frame.render_widget(
                Paragraph::new(format!("\n\n  {}", msg))
                    .block(
                        Block::bordered()
                            .title(" Status ")
                            .border_type(BorderType::Rounded)
                            .fg(theme.border_normal)
                            .bg(theme.content_bg),
                    )
                    .alignment(Alignment::Center),
                area,
            );
            return;
        }
    }

    if app.mode == AppMode::SshForm {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let labels = [
            " Server Name (e.g. Prod-1) ",
            " Host IP or Domain ",
            " Port ",
            " SSH Username ",
            " Password OR Path to SSH Key (~/.ssh/id_rsa) ",
        ];

        for i in 0..5 {
            let color = if app.ssh_form.active_field == i {
                theme.border_focused
            } else {
                theme.border_normal
            };
            let val = app.ssh_form.inputs[i].value();
            let mut display_val = if val.is_empty() { " " } else { val }.to_string();
            if app.ssh_form.active_field == i {
                display_val = format!("{}_", display_val);
            }
            frame.render_widget(
                Paragraph::new(display_val).block(
                    Block::bordered()
                        .title(labels[i])
                        .border_type(BorderType::Rounded)
                        .fg(color)
                        .bg(theme.content_bg),
                ),
                chunks[i],
            );
        }
        frame.render_widget(
            Paragraph::new("\n [↑↓/Tab] Navigate  [↵] Save  [Esc] Cancel")
                .style(Style::default().fg(theme.border_dim)),
            chunks[5],
        );
        return;
    }

    if app.mode == AppMode::ServerSelect {
        let scheme_name = app.color_scheme.name();
        let welcome_text = vec![
            Line::from(""),
            Line::from(" Welcome to Logia-rs!").style(
                Style::default()
                    .fg(theme.welcome_title)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from(" TUI for viewing and searching logs."),
            Line::from(" laravel and system oriented"),
            Line::from(""),
            Line::from(" Features:").style(Style::default().fg(theme.welcome_feature_heading)),
            Line::from("  • Connect to remote servers via SSH and keep configs"),
            Line::from("  • Navigate the filesystem and scan directories for logs."),
            Line::from(""),
            Line::from(" How to start:").style(Style::default().fg(theme.welcome_how_heading)),
            Line::from("  1. Select a server (Local or Remote) from the sidebar."),
            Line::from("  2. Press Enter to connect and explore logs."),
            Line::from("  3. Press 'a' or 'n' to add a new SSH server."),
            Line::from("  4. Press 't' to cycle themes."),
            Line::from("  5. Press 'q' or Esc to quit."),
            Line::from(""),
            Line::from(format!("  Theme: {}", scheme_name))
                .style(Style::default().fg(theme.border_dim)),
        ];

        frame.render_widget(
            Paragraph::new(welcome_text)
                .block(
                    Block::bordered()
                        .title(" Getting Started ")
                        .border_type(BorderType::Rounded)
                        .fg(theme.border_normal)
                        .bg(theme.content_bg),
                )
                .alignment(Alignment::Left),
            area,
        );
        return;
    }

    if app.mode == AppMode::GotoPrompt {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let val = app.goto_input.value();
        let display = format!("{}_", val);

        // Show matching completions below the input
        let completions = crate::app::tab_complete(val)
            .map(|c| format!("  → {}", c))
            .unwrap_or_default();

        frame.render_widget(
            Paragraph::new(display).block(
                Block::bordered()
                    .title(" Go to path (Tab to autocomplete) ")
                    .border_type(BorderType::Rounded)
                    .fg(theme.border_focused)
                    .bg(theme.content_bg),
            ),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new(completions)
                .block(
                    Block::bordered()
                        .title(" Suggestion ")
                        .border_type(BorderType::Rounded)
                        .fg(theme.border_dim)
                        .bg(theme.content_bg),
                )
                .fg(theme.item_local),
            chunks[1],
        );
        return;
    }

    if app.mode == AppMode::FileExplorer || app.mode == AppMode::SearchPrompt {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);
        let search_color = if app.mode == AppMode::SearchPrompt {
            theme.border_focused
        } else {
            theme.border_normal
        };
        let val = app.search_input.value();
        let display_val = if val.is_empty() && app.mode != AppMode::SearchPrompt {
            " Press '/' to search...".to_string()
        } else {
            val.to_string()
        };
        frame.render_widget(
            Paragraph::new(display_val).block(
                Block::bordered()
                    .title(" Finder (Fuzzy) ")
                    .border_type(BorderType::Rounded)
                    .fg(search_color)
                    .bg(theme.content_bg),
            ),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new("\n\n  Select a log file from the sidebar to open it.\n\n  Use '-' or Backspace to navigate up, 's' to scan a directory.")
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .fg(theme.border_dim)
                        .bg(theme.content_bg),
                )
                .fg(theme.item_normal),
            chunks[1],
        );
        return;
    }

    // Log viewer
    let has_export_msg = app.export_message.is_some();
    let (log_area, export_area) = if has_export_msg {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let file_name = app
        .active_log
        .as_ref()
        .map(|l| l.source.name.clone())
        .unwrap_or_else(|| "Logs".to_string());

    let scroll = app.active_log.as_ref().map(|l| l.scroll).unwrap_or(0);

    let lines: Vec<Line> = if let Some(log) = &app.active_log {
        let query = app.search_input.value().to_lowercase();
        log.lines
            .iter()
            .filter(|line_str| query.is_empty() || line_str.to_lowercase().contains(&query))
            .map(|line_str| parse_laravel_line(line_str, theme))
            .collect()
    } else {
        vec![]
    };

    let title = if app.mode == AppMode::LogSearchPrompt || !app.search_input.value().is_empty() {
        format!(" {} — Search: {}_ ", file_name, app.search_input.value())
    } else {
        format!(" {} — [/] Search  [r] Refresh  [e] Export ", file_name)
    };

    let border_color = if app.mode == AppMode::LogSearchPrompt {
        theme.border_focused
    } else {
        theme.border_normal
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(title)
                    .border_type(BorderType::Rounded)
                    .fg(border_color)
                    .bg(theme.content_bg),
            )
            .fg(theme.log_default)
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false }),
        log_area,
    );

    if let (Some(msg), Some(export_rect)) = (&app.export_message, export_area) {
        let is_error = msg.starts_with("Export failed");
        let fg = if is_error { Color::Red } else { Color::Green };
        frame.render_widget(
            Paragraph::new(format!(" {}", msg))
                .style(Style::default().fg(fg).add_modifier(Modifier::BOLD)),
            export_rect,
        );
    }
}
