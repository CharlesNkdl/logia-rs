use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Stylize};
use ratatui::widgets::{Block, BorderType, Widget};

pub fn render_header(area: Rect, buf: &mut Buffer) {
    Block::bordered()
        .title(" logia-rs ")
        .border_type(BorderType::Rounded)
        .fg(Color::Cyan)
        .bg(Color::Black)
        .render(area, buf);
}
