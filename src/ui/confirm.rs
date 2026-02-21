use crossterm::event::{ KeyCode, KeyEvent };
use ratatui::{
    Frame,
    layout::{ Alignment, Constraint, Direction, Layout, Rect },
    prelude::Stylize,
    style::{ Color, Style },
    text::Line,
    widgets::{ Block, BorderType, Borders, Clear, Paragraph },
};

#[derive(Clone)]
pub struct ConfirmPopup {
    pub title: String,
    pub message: String,
}

impl ConfirmPopup {
    pub fn new(title: String, message: String) -> Self {
        Self {
            title,
            message,
        }
    }

    pub fn handle_key(&self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(true),
            _ => Some(false),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create a centered area for the popup
        let popup_area = self.centered_rect(60, 20, area);
        // Clear the background so the list doesn't bleed through
        f.render_widget(Clear, popup_area);

        let block = Block::bordered()
            .title(self.title.as_str())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red));

        let text = vec![
            Line::from(self.message.as_str()),
            Line::from(""),
            Line::from("(Y)es  /  (N)o".bold())
        ];

        let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Center);

        f.render_widget(paragraph, popup_area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
