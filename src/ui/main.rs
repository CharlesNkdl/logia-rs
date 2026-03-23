use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Widget};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType};

pub fn render_main(area: Rect, buf: &mut Buffer) {
    Block::bordered()
        .title(" Logs ")
        .border_type(BorderType::Rounded)
        .fg(Color::Gray)
        .bg(Color::Black)
        .render(area, buf);
}
