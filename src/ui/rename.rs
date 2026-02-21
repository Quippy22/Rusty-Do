use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, palette::tailwind},
    text::Span,
    widgets::{Block, BorderType, Clear, Paragraph},
};

#[derive(Clone)]
pub struct RenamePopup {
    pub title: String,
    pub warning: String,
    pub input: String,
    pub is_first_input: bool,
}

impl RenamePopup {
    pub fn new(title: String, initial_value: String, warning: Option<String>) -> Self {
        Self {
            title,
            input: initial_value,
            warning: {
                match warning {
                    Some(warning) => warning,
                    None => "".to_string(),
                }
            },
            is_first_input: true,
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
                self.is_first_input = false;
                self.input.pop();
                None
            }
            KeyCode::Char(c) => {
                if self.is_first_input {
                    self.input.clear();
                    self.is_first_input = false;
                }
                self.input.push(c);
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

        // 2. Cannot contain illegal filename characters
        let illegal_chars = ['/', '\\', '<', '>', ':', '"', '|', '?', '*'];
        if trimmed.chars().any(|c| illegal_chars.contains(&c)) {
            self.warning = "Name cannot contain illegal characters.".to_string();
            return false;
        }

        // 3. Cannot be too long
        if trimmed.len() > 200 {
            self.warning = "Name is too long.".to_string();
            return false;
        }

        self.warning = "".to_string();
        true
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // 1. Position the box (using the same size as your reshaped delete popup)
        let popup_area = self.centered_rect(50, 4, area);

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
                Constraint::Length(1), // Warning
                Constraint::Min(0),    // Buffer space
                Constraint::Length(1), // The actual input line
            ])
            .split(inner_area);

        // 5. Render the input string with a "cursor"
        // We append a '█' to the end of the string to simulate a cursor
        let input_display = if self.is_first_input {
            Span::styled(
                self.input.as_str(),
                Style::default().bg(Color::White).fg(Color::Black),
            )
        } else {
            Span::raw(format!("{}█", self.input))
        };
        let input_p = Paragraph::new(input_display).alignment(Alignment::Center);
        f.render_widget(input_p, chunks[2]);

        // 6. Add the warning
        let warning = Paragraph::new(self.warning.to_string())
            .style(Style::default().fg(tailwind::AMBER.c500));
        f.render_widget(warning, chunks[0]);
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
