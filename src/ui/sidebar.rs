use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Color;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, BorderType, List, ListItem};

pub fn render_sidebar(frame: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .sources
        .iter()
        .map(|s| ListItem::new(s.name.clone()).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Sources ")
                .border_type(BorderType::Rounded)
                .fg(Color::Gray)
                .bg(Color::Black),
        )
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol("▶ ");

    // render_stateful_widget permet à List de gérer la sélection et le scroll
    frame.render_stateful_widget(list, area, &mut app.list_state);
}
