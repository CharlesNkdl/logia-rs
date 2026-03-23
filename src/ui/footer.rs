use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Stylize};
use ratatui::widgets::{Block, BorderType, Paragraph};

pub fn render_footer(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(" ↑↓ Navigate   ↵ Open   q Quit")
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .fg(Color::Gray)
                    .bg(Color::Black),
            )
            .fg(Color::DarkGray),
        area,
    );
}
