use crate::app::App;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Color;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType, List, ListItem, Widget};

pub fn render_sidebar(area: Rect, buf: &mut Buffer, app: &App) {
    let items: Vec<ListItem> = app
        .sources
        .iter()
        .map(|s| ListItem::new(s.name.clone()))
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Sources ")
                .border_type(BorderType::Rounded)
                .fg(Color::Gray)
                .bg(Color::Black),
        )
        .fg(Color::White);

    list.render(area, buf);
}
