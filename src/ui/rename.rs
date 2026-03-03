use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::actions::PendingAction;
use crate::ui::theme::theme;

#[derive(Clone)]
pub struct RenamePopup {
    pub title: String,
    pub input: String,
    pub cursor_pos: usize,
    pub is_first_input: bool, // Clear on first char
}

impl RenamePopup {
    pub fn new(title: String, initial_val: String, action: PendingAction) -> Self {
        let is_first = matches!(
            action,
            PendingAction::AddSubtaskBefore | PendingAction::AddSubtaskAfter
        );
        let cursor_pos = initial_val.len();
        Self {
            title,
            input: initial_val,
            cursor_pos,
            is_first_input: is_first,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Enter => Some(true),
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

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let width = 50;
        let height = 5;
        let popup_area = self.centered_rect(width, height, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme().border_focused))
            .title(format!(" {} ", self.title))
            .title_alignment(Alignment::Center);

        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Input
                Constraint::Min(0),    // Padding
            ])
            .split(inner_area);

        let input_line = if self.input.is_empty() {
            Line::from(vec![Span::styled(
                "▎",
                Style::default().fg(Color::White),
            )])
        } else {
            let prefix = &self.input[..self.cursor_pos];
            let suffix = &self.input[self.cursor_pos..];
            Line::from(vec![
                Span::raw(prefix),
                Span::styled("▎", Style::default().fg(Color::White)),
                Span::raw(suffix),
            ])
        };

        f.render_widget(Paragraph::new(input_line).alignment(Alignment::Center), chunks[1]);
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
