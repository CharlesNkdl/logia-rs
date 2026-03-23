use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Stylize};
use ratatui::widgets::{Block, BorderType};

pub fn render_header(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Block::bordered()
            .title(" logia-rs ")
            .border_type(BorderType::Rounded)
            .fg(Color::Cyan)
            .bg(Color::Black),
        area,
    );
}
