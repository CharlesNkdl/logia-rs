use crate::app::{App, AppMode};
use crate::ui::footer::render_footer;
use crate::ui::header::render_header;
use crate::ui::main::render_main;
use crate::ui::sidebar::render_sidebar;
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = Theme::from_scheme(&app.color_scheme);
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(vertical[1]);
    render_header(frame, vertical[0], &theme);
    match app.mode {
        AppMode::ServerSelect | AppMode::SshForm => {
            render_sidebar(frame, horizontal[0], app, &theme);
            render_main(frame, horizontal[1], app, &theme);
        }
        AppMode::FileExplorer | AppMode::SearchPrompt | AppMode::GotoPrompt => {
            render_sidebar(frame, horizontal[0], app, &theme);
            render_main(frame, horizontal[1], app, &theme);
        }
        AppMode::LogViewer | AppMode::LogSearchPrompt => {
            render_main(frame, vertical[1], app, &theme);
        }
    }
    render_footer(frame, vertical[2], app, &theme);
}
