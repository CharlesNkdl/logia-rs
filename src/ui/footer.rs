use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Stylize};
use ratatui::widgets::{Block, BorderType, Widget};

pub fn render_footer(area: Rect, buf: &mut Buffer) {
    Block::bordered()
        .title(" Raccourcis ")
        .border_type(BorderType::Rounded)
        .fg(Color::Gray)
        .bg(Color::Black)
        .render(area, buf);
}
