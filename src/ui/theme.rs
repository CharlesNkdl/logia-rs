use ratatui::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ColorScheme {
    #[default]
    Default,
    HighContrast,
    Dark,
}

impl ColorScheme {
    pub fn next(&self) -> Self {
        match self {
            ColorScheme::Default => ColorScheme::HighContrast,
            ColorScheme::HighContrast => ColorScheme::Dark,
            ColorScheme::Dark => ColorScheme::Default,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ColorScheme::Default => "Default",
            ColorScheme::HighContrast => "HighContrast",
            ColorScheme::Dark => "Dark",
        }
    }
}

/// All colors used across the TUI, derived from a `ColorScheme`.
pub struct Theme {
    // Borders
    pub border_focused: Color,
    pub border_normal: Color,
    pub border_dim: Color,

    // List / selection
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub item_local: Color,
    pub item_normal: Color,

    // Header / footer
    pub header_fg: Color,
    pub header_bg: Color,
    pub footer_fg: Color,
    pub footer_bg: Color,
    pub footer_text: Color,

    // Welcome screen
    pub welcome_title: Color,
    pub welcome_feature_heading: Color,
    pub welcome_how_heading: Color,

    // Log level colors
    pub log_info: Color,
    pub log_warn: Color,
    pub log_error: Color,
    pub log_debug: Color,
    pub log_env: Color,
    pub log_date: Color,
    pub log_default: Color,

    // Background used for content areas (Reset = transparent, explicit = opaque)
    pub content_bg: Color,
}

impl Theme {
    pub fn from_scheme(scheme: &ColorScheme) -> Self {
        match scheme {
            ColorScheme::Default => Self {
                border_focused: Color::LightBlue,
                border_normal: Color::Gray,
                border_dim: Color::DarkGray,

                highlight_fg: Color::Black,
                highlight_bg: Color::LightCyan,
                item_local: Color::Yellow,
                item_normal: Color::White,

                header_fg: Color::Cyan,
                header_bg: Color::Reset,
                footer_fg: Color::Gray,
                footer_bg: Color::Reset,
                footer_text: Color::Gray,

                welcome_title: Color::LightCyan,
                welcome_feature_heading: Color::LightYellow,
                welcome_how_heading: Color::LightGreen,

                log_info: Color::LightGreen,
                log_warn: Color::LightYellow,
                log_error: Color::LightRed,
                log_debug: Color::LightCyan,
                log_env: Color::LightBlue,
                log_date: Color::DarkGray,
                log_default: Color::White,

                content_bg: Color::Reset,
            },
            ColorScheme::HighContrast => Self {
                border_focused: Color::Yellow,
                border_normal: Color::White,
                border_dim: Color::Gray,

                highlight_fg: Color::Black,
                highlight_bg: Color::Yellow,
                item_local: Color::Cyan,
                item_normal: Color::White,

                header_fg: Color::White,
                header_bg: Color::Blue,
                footer_fg: Color::White,
                footer_bg: Color::Blue,
                footer_text: Color::White,

                welcome_title: Color::Cyan,
                welcome_feature_heading: Color::Yellow,
                welcome_how_heading: Color::Green,

                log_info: Color::Green,
                log_warn: Color::Yellow,
                log_error: Color::Red,
                log_debug: Color::Cyan,
                log_env: Color::Blue,
                log_date: Color::Gray,
                log_default: Color::White,

                content_bg: Color::Reset,
            },
            ColorScheme::Dark => Self {
                border_focused: Color::Rgb(100, 180, 255),
                border_normal: Color::Rgb(100, 100, 120),
                border_dim: Color::Rgb(60, 60, 70),

                highlight_fg: Color::Rgb(20, 20, 30),
                highlight_bg: Color::Rgb(100, 180, 255),
                item_local: Color::Rgb(255, 200, 80),
                item_normal: Color::Rgb(200, 200, 210),

                header_fg: Color::Rgb(100, 180, 255),
                header_bg: Color::Rgb(20, 20, 30),
                footer_fg: Color::Rgb(100, 100, 120),
                footer_bg: Color::Rgb(20, 20, 30),
                footer_text: Color::Rgb(140, 140, 160),

                welcome_title: Color::Rgb(100, 180, 255),
                welcome_feature_heading: Color::Rgb(255, 200, 80),
                welcome_how_heading: Color::Rgb(100, 220, 130),

                log_info: Color::Rgb(100, 220, 130),
                log_warn: Color::Rgb(255, 200, 80),
                log_error: Color::Rgb(255, 90, 90),
                log_debug: Color::Rgb(100, 180, 255),
                log_env: Color::Rgb(160, 130, 255),
                log_date: Color::Rgb(100, 100, 120),
                log_default: Color::Rgb(200, 200, 210),

                content_bg: Color::Rgb(20, 20, 30),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schemes_produce_different_colors() {
        let default = Theme::from_scheme(&ColorScheme::Default);
        let high_contrast = Theme::from_scheme(&ColorScheme::HighContrast);
        assert!(
            default.border_focused != high_contrast.border_focused
                || default.highlight_bg != high_contrast.highlight_bg
                || default.header_bg != high_contrast.header_bg
                || default.log_error != high_contrast.log_error,
            "Default and HighContrast schemes must differ in at least one color"
        );
    }
}
