use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, palette::tailwind},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::app::actions::PendingAction;

#[derive(Clone)]
pub struct RenamePopup {
    pub title: String,
    pub warning: String,
    pub input: String,
    pub is_first_input: bool,
    pub cursor_pos: usize,
    pub action: PendingAction,
}

impl RenamePopup {
    pub fn new(title: String, initial_value: String, action: PendingAction) -> Self {
        let len = initial_value.len();
        Self {
            title,
            cursor_pos: len,
            input: initial_value,
            warning: String::new(),
            is_first_input: true,
            action,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Enter => {
                if self.is_valid() {
                    Some(true)
                } else {
                    None
                }
            }
            KeyCode::Esc => Some(false),
            KeyCode::Backspace => {
                if self.is_first_input {
                    self.input.clear();
                    self.cursor_pos = 0;
                    self.is_first_input = false;
                } else if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
                None
            }
            KeyCode::Delete => {
                if self.is_first_input {
                    self.input.clear();
                    self.cursor_pos = 0;
                    self.is_first_input = false;
                } else if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                }
                None
            }
            KeyCode::Left => {
                if self.is_first_input {
                    self.is_first_input = false;
                    self.cursor_pos = 0;
                } else if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.is_first_input {
                    self.is_first_input = false;
                    self.cursor_pos = self.input.len();
                } else if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                None
            }
            KeyCode::Home => {
                self.is_first_input = false;
                self.cursor_pos = 0;
                None
            }
            KeyCode::End => {
                self.is_first_input = false;
                self.cursor_pos = self.input.len();
                None
            }
            KeyCode::Char(c) => {
                if self.is_first_input {
                    self.input.clear();
                    self.cursor_pos = 0;
                    self.is_first_input = false;
                }
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                None
            }
            _ => None,
        }
    }

    pub fn is_valid(&mut self) -> bool {
        let trimmed = self.input.trim();

        // 1. Cannot be empty
        if trimmed.is_empty() {
            self.warning = "Name cannot be empty.".to_string();
            return false;
        }

        // 2. Rules for Notebooks (Filenames)
        if matches!(self.action, PendingAction::RenameNotebook | PendingAction::AddNotebook) {
            let illegal_chars = ['/', '\\', '<', '>', ':', '"', '|', '?', '*'];
            if trimmed.chars().any(|c| illegal_chars.contains(&c)) {
                self.warning = "Notebook names cannot contain illegal characters (/\\<>:\"|?*).".to_string();
                return false;
            }
        }

        // 3. Length check
        if trimmed.len() > 200 {
            self.warning = "Name is too long.".to_string();
            return false;
        }

        self.warning = "".to_string();
        true
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // 1. Calculate dynamic width (min 50, max 80% of screen)
        let text_len = self.input.len() as u16 + 10; // +10 for some padding
        let max_allowed = (area.width as f32 * 0.8) as u16;
        let width = text_len.clamp(50, max_allowed);
        
        let popup_area = self.centered_rect(width, 5, area);

        // 2. Clear the background
        f.render_widget(Clear, popup_area);

        // 3. Style the block
        let block = Block::bordered()
            .title(self.title.as_str())
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue));

        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        // 4. Split the inner area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Warning
                Constraint::Min(0),    // Buffer
                Constraint::Length(1), // Input
            ])
            .split(inner_area);

        // 5. Render Input with Cursor
        let input_line = if self.is_first_input {
            Line::from(vec![
                Span::styled(
                    &self.input,
                    Style::default().bg(Color::White).fg(Color::Black),
                )
            ])
        } else {
            let prefix = &self.input[..self.cursor_pos];
            let suffix = &self.input[self.cursor_pos..];
            Line::from(vec![
                Span::raw(prefix),
                Span::styled("▎", Style::default().fg(Color::White)),
                Span::raw(suffix),
            ])
        };

        f.render_widget(Paragraph::new(input_line).alignment(Alignment::Center), chunks[2]);

        // 6. Render Warning
        if !self.warning.is_empty() {
            let warning = Paragraph::new(self.warning.as_str())
                .style(Style::default().fg(tailwind::AMBER.c500))
                .alignment(Alignment::Center);
            f.render_widget(warning, chunks[0]);
        }
    }

    fn centered_rect(&self, width: u16, height: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(height),
                Constraint::Min(0),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(width),
                Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    }
}
