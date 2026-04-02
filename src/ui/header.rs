use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType};

pub fn render_header(frame: &mut Frame, area: Rect, theme: &Theme) {
    frame.render_widget(
        Block::bordered()
            .title(" logia-rs ")
            .border_type(BorderType::Rounded)
            .fg(theme.header_fg)
            .bg(theme.header_bg),
        area,
    );
}
