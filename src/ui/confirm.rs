use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

#[derive(Clone)]
pub struct ConfirmPopup {
    pub title: String,
    pub message: String,
}

impl ConfirmPopup {
    pub fn new(title: String, message: String) -> Self {
        Self { title, message }
    }

    pub fn handle_key(&self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(true),
            _ => Some(false),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Fix size: 40 columns wide, 6 lines tall
        let popup_area = self.centered_rect(40, 6, area);
        f.render_widget(Clear, popup_area);

        let block = Block::bordered()
            .title(self.title.as_str())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red));

        // 1. Get the area inside the borders
        let inner_area = block.inner(popup_area);

        // 2. Draw the borders to the screen first
        f.render_widget(block, popup_area);

        // 3. Split the inside space: Top gets everything, Bottom gets 1 line
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // This pushes the bottom constraint down
                Constraint::Length(1), // Exactly 1 line for the Yes/No
            ])
            .split(inner_area);

        let message = Paragraph::new(self.message.as_str())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let buttons = Paragraph::new("(Y)es  /  (N)o".bold()).alignment(Alignment::Center);

        // 4. Render the text into their respective layout chunks
        f.render_widget(message, inner_layout[0]);
        f.render_widget(buttons, inner_layout[1]);
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
