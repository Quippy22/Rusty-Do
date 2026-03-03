use std::path::PathBuf;
use std::sync::OnceLock;
use ratatui::style::Color;
use ratatui::style::palette::tailwind;
use serde::{Deserialize, Serialize};

static GLOBAL_THEME: OnceLock<Theme> = OnceLock::new();

/// Access the global theme. Initializes with defaults if not set.
pub fn theme() -> &'static Theme {
    GLOBAL_THEME.get_or_init(Theme::default)
}

/// Initialize the global theme with a custom configuration.
/// Should be called once at application startup.
pub fn init_theme(custom: Theme) {
    let _ = GLOBAL_THEME.set(custom);
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub title_main: Color,
    pub title_secondary: Color,
    pub highlight: Color,
    pub completion_done: Color,
    pub completion_pending: Color,
    pub help_text: Color,
    pub warning: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border_focused: tailwind::ROSE.c500,
            border_unfocused: tailwind::WHITE,
            title_main: tailwind::SKY.c400,
            title_secondary: tailwind::AMBER.c400,
            highlight: tailwind::ROSE.c500,
            completion_done: tailwind::GREEN.c400,
            completion_pending: tailwind::WHITE,
            help_text: tailwind::GRAY.c500,
            warning: tailwind::AMBER.c500,
        }
    }
}

/// Serializable configuration for user-provided themes.
#[derive(Serialize, Deserialize, Default)]
pub struct ThemeConfig {
    pub name: String,
    pub border_focused: Option<String>,
    pub border_unfocused: Option<String>,
    pub title_main: Option<String>,
    pub title_secondary: Option<String>,
    pub highlight: Option<String>,
    pub completion_done: Option<String>,
    pub completion_pending: Option<String>,
    pub help_text: Option<String>,
    pub warning: Option<String>,
}

impl Theme {
    pub fn load(path: PathBuf) -> Self {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<ThemeConfig>(&content) {
                return Self::from_config(config);
            }
        }
        Self::default()
    }

    pub fn from_config(config: ThemeConfig) -> Self {
        let mut theme = Self::default();

        if let Some(c) = config.border_focused.and_then(|s| parse_color(&s)) { theme.border_focused = c; }
        if let Some(c) = config.border_unfocused.and_then(|s| parse_color(&s)) { theme.border_unfocused = c; }
        if let Some(c) = config.title_main.and_then(|s| parse_color(&s)) { theme.title_main = c; }
        if let Some(c) = config.title_secondary.and_then(|s| parse_color(&s)) { theme.title_secondary = c; }
        if let Some(c) = config.highlight.and_then(|s| parse_color(&s)) { theme.highlight = c; }
        if let Some(c) = config.completion_done.and_then(|s| parse_color(&s)) { theme.completion_done = c; }
        if let Some(c) = config.completion_pending.and_then(|s| parse_color(&s)) { theme.completion_pending = c; }
        if let Some(c) = config.help_text.and_then(|s| parse_color(&s)) { theme.help_text = c; }
        if let Some(c) = config.warning.and_then(|s| parse_color(&s)) { theme.warning = c; }

        theme
    }
}

fn parse_color(s: &str) -> Option<Color> {
    if s.starts_with('#') {
        let hex = s.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Some(Color::Rgb(r, g, b));
            }
        }
    }

    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::Gray),
        "white" => Some(Color::White),
        _ => None,
    }
}
