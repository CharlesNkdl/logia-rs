use crate::ui::theme::Theme;
use ratatui::prelude::{Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use std::sync::LazyLock;

// Example Laravel log:
// [2024-04-02 12:00:00] local.INFO: Some message here

static LARAVEL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<date>[^\]]+)\] (?P<env>\w+)\.(?P<level>[A-Z]+): (?P<msg>.*)$").unwrap()
});

pub fn parse_laravel_line(input: &str, theme: &Theme) -> Line<'static> {
    if let Some(caps) = LARAVEL_REGEX.captures(input) {
        let date = caps.name("date").map_or("", |m| m.as_str());
        let env = caps.name("env").map_or("", |m| m.as_str());
        let level = caps.name("level").map_or("", |m| m.as_str());
        let msg = caps.name("msg").map_or("", |m| m.as_str());
        let level_color = match level {
            "INFO" => theme.log_info,
            "WARNING" => theme.log_warn,
            "ERROR" | "CRITICAL" | "EMERGENCY" => theme.log_error,
            "DEBUG" => theme.log_debug,
            _ => theme.log_default,
        };

        Line::from(vec![
            Span::styled(format!("[{}] ", date), Style::default().fg(theme.log_date)),
            Span::styled(format!("{}.", env), Style::default().fg(theme.log_env)),
            Span::styled(
                format!("{}: ", level),
                Style::default()
                    .fg(level_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(msg.to_string()),
        ])
    } else {
        Line::from(Span::raw(input.to_string()))
    }
}
