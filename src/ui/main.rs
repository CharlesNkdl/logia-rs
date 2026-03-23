use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Color;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType, Paragraph};

pub fn render_main(frame: &mut Frame, area: Rect, app: &App) {
    let title = app
        .active_log
        .as_ref()
        .map(|l| format!(" {} ", l.source.name))
        .unwrap_or(" Logs ".to_string());

    let content = app
        .active_log
        .as_ref()
        .map(|l| l.lines.join("\n"))
        .unwrap_or("↵  Select a log file to read".to_string());

    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::bordered()
                    .title(title)
                    .border_type(BorderType::Rounded)
                    .fg(Color::Gray)
                    .bg(Color::Black),
            )
            .fg(Color::White),
        area,
    );
}
