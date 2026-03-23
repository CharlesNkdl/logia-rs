use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;
use crate::ui::footer::render_footer;
use crate::ui::header::render_header;
use crate::ui::main::render_main;
use crate::ui::sidebar::render_sidebar;

pub fn render(frame: &mut Frame, app: &mut App) {
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
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(vertical[1]);

    render_header(frame, vertical[0]);
    render_sidebar(frame, horizontal[0], app);
    render_main(frame, horizontal[1], app);
    render_footer(frame, vertical[2]);
}
