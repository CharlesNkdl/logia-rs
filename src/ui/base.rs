use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::app::App;
use crate::ui::footer::render_footer;
use crate::ui::header::render_header;
use crate::ui::main::render_main;
use crate::ui::sidebar::render_sidebar;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(0),    // body
                Constraint::Length(3), // footer
            ])
            .split(area);

        let header_area = vertical[0];
        let body_area = vertical[1];
        let footer_area = vertical[2];

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)])
            .split(body_area);

        let sidebar_area = horizontal[0];
        let main_area = horizontal[1];

        render_header(header_area, buf);
        render_sidebar(sidebar_area, buf, self);
        render_main(main_area, buf);
        render_footer(footer_area, buf);
    }
}
