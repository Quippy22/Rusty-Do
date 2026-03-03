use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use ratatui::style::Color;
use ratatui::style::palette::tailwind;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

// Use Lazy and RwLock to allow live theme swapping
static GLOBAL_THEME: Lazy<Arc<RwLock<Theme>>> = Lazy::new(|| Arc::new(RwLock::new(Theme::default())));

/// Access the global theme for reading.
pub fn theme() -> Theme {
    GLOBAL_THEME.read().unwrap().clone()
}

/// Update the global theme live.
pub fn set_theme(new_theme: Theme) {
    if let Ok(mut lock) = GLOBAL_THEME.write() {
        *lock = new_theme;
    }
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

impl Theme {
    pub fn nord() -> Self {
        Self {
            border_focused: Color::Rgb(136, 192, 208),
            border_unfocused: Color::Rgb(76, 86, 106),
            title_main: Color::Rgb(143, 188, 187),
            title_secondary: Color::Rgb(235, 203, 139),
            highlight: Color::Rgb(136, 192, 208),
            completion_done: Color::Rgb(163, 190, 140),
            completion_pending: Color::Rgb(216, 222, 233),
            help_text: Color::Rgb(129, 161, 193),
            warning: Color::Rgb(208, 135, 112),
        }
    }

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
