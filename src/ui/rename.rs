use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Clear, Paragraph},
};

#[derive(Clone)]
pub struct RenamePopup {
    pub title: String,
    pub input: String,
}

impl RenamePopup {
    pub fn new(title: String, initial_value: String) -> Self {
        Self {
            title,
            input: initial_value,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Enter => Some(true),
            KeyCode::Esc => Some(false),
            KeyCode::Backspace => {
                self.input.pop();
                None
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                None
            }
            _ => None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // 1. Position the box (using the same size as your reshaped delete popup)
        let popup_area = self.centered_rect(40, 6, area);

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

        // 4. Split the inner area so we can have a label and the input line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Message/Label
                Constraint::Min(0),    // Buffer space
                Constraint::Length(1), // The actual input line
            ])
            .split(inner_area);

        // 5. Render the input string with a "cursor"
        // We append a '█' to the end of the string to simulate a cursor
        let input_display = format!("{}_", self.input);
        let input_p = Paragraph::new(input_display).alignment(Alignment::Center);

        f.render_widget(input_p, chunks[2]);
    }

    // Reuse your centered_rect logic here
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
