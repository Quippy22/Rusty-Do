use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};

#[derive(Clone)]
pub struct ConfirmPopup {
    pub title: String,
    pub message: String,
    pub buttons: Vec<String>,
    pub selected_idx: usize,
}

impl ConfirmPopup {
    pub fn new(title: String, message: String, buttons: Vec<String>) -> Self {
        Self {
            title,
            message,
            buttons,
            selected_idx: 0,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<usize> {
        match key.code {
            KeyCode::Enter => Some(self.selected_idx),
            KeyCode::Esc => None,
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.selected_idx < self.buttons.len() - 1 {
                    self.selected_idx += 1;
                }
                None
            }
            KeyCode::Char(c) => {
                // Check if any button starts with this character (Shortcut logic)
                let target = c.to_lowercase().next()?;
                for (i, btn) in self.buttons.iter().enumerate() {
                    if btn.to_lowercase().starts_with(target) {
                        return Some(i);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let width = (self.buttons.len() as u16 * 18).max(45).min(area.width);
        let height = 8;
        let popup_area = self.centered_rect(width, height, area);
        f.render_widget(Clear, popup_area);

        let block = Block::bordered()
            .title(self.title.as_str())
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red));

        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Buttons
            ])
            .margin(1)
            .split(inner_area);

        // 1. Render Message
        f.render_widget(
            Paragraph::new(self.message.as_str())
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            layout[0],
        );

        // 2. Render Buttons using Flex
        let button_constraints = vec![Constraint::Length(15); self.buttons.len()];
        let button_chunks = Layout::horizontal(button_constraints)
            .flex(Flex::Center)
            .split(layout[2]);

        for (i, btn) in self.buttons.iter().enumerate() {
            let display_name = self.format_button_label(btn);
            let style = if i == self.selected_idx {
                Style::default().bg(Color::White).fg(Color::Black).bold()
            } else {
                Style::default().fg(Color::Gray)
            };

            f.render_widget(
                Paragraph::new(display_name).alignment(Alignment::Center).style(style),
                button_chunks[i],
            );
        }
    }

    fn format_button_label<'a>(&self, label: &'a str) -> Line<'a> {
        if let Some(first) = label.chars().next() {
            let rest = &label[first.len_utf8()..];
            Line::from(vec![
                Span::raw("("),
                Span::raw(first.to_string()).bold().underlined(),
                Span::raw(")"),
                Span::raw(rest),
            ])
        } else {
            Line::from(label)
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
