use crate::app::{App, AppMode};
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, Paragraph};

pub fn render_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let hints = match app.mode {
        AppMode::ServerSelect => " ↑↓ Navigate   ↵ Connect   a New SSH   q Quit   t Theme",
        AppMode::SshForm => " ↑↓/Tab Navigate fields   ↵ Save   Esc Cancel",
        AppMode::FileExplorer => " ↑↓ Navigate   ↵ Open   / Search   g Goto path   - Parent   s Scan   Esc Back   t Theme",
        AppMode::SearchPrompt => " Type to filter   ↵ Confirm   Esc Cancel",
        AppMode::GotoPrompt => " Type absolute path   Tab Autocomplete   ↵ Go   Esc Cancel",
        AppMode::LogViewer => " ↑↓/j/k Scroll   PgUp/PgDn   Home/End   / Search   r Refresh   e Export   Esc Back",
        AppMode::LogSearchPrompt => " Type to search   ↵ Confirm   Esc Cancel",
    };

    frame.render_widget(
        Paragraph::new(hints)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .fg(theme.footer_fg)
                    .bg(theme.footer_bg),
            )
            .fg(theme.footer_text),
        area,
    );
}
