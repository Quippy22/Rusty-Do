use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Buffer, Rect},
    style::{Modifier, Style, palette::tailwind},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
        Wrap,
    },
};

use crate::models::task::Task;

#[derive(Clone)]
pub struct TaskColumnState {
    pub state: ListState,
}

pub struct TaskColumn<'a> {
    pub task: &'a Task,
    pub is_focused: bool,
}

impl<'a> TaskColumn<'a> {
    pub fn new(task: &'a Task, is_focused: bool) -> Self {
        Self { task, is_focused }
    }
}

impl<'a> StatefulWidget for TaskColumn<'a> {
    type State = TaskColumnState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_color = if self.is_focused {
            tailwind::ROSE.c500
        } else {
            tailwind::WHITE
        };

        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let inner_area = main_block.inner(area);
        main_block.render(area, buf);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(4),    // Header
                Constraint::Length(1), // Separator line
                Constraint::Min(0),    // List of subtasks
            ])
            .split(inner_area);

        // -- The Header --
        let header = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(inner_chunks[0]);

        let title = Paragraph::new(self.task.name.to_string())
            .centered()
            .style(
                Style::default()
                    .fg(tailwind::SKY.c400)
                    .add_modifier(Modifier::BOLD),
            )
            .wrap(Wrap { trim: true });
        title.render(header[0], buf);

        let completion = String::from(format!("Completion {}", self.task.completion));
        let completion_color = if self.task.is_done {
            tailwind::GREEN.c400
        } else {
            tailwind::WHITE
        };
        Paragraph::new(completion)
            .style(Style::default().fg(completion_color))
            .render(header[2], buf);

        // -- The Separator --
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(border_color))
            .render(inner_chunks[1], buf);

        // -- The Body --
        let items: Vec<ListItem> = self
            .task
            .subtasks
            .iter()
            .map(|s| {
                let symbol = if s.is_done { "[x]" } else { "[ ]" };
                let full_text = format!("{} {}", symbol, s.name);
                
                // Wrap text to fit inside the column (max 70 - 4 for borders/padding)
                let wrapped = wrap_text(&full_text, 66);
                ListItem::new(wrapped)
            })
            .collect();

        let list = List::new(items).highlight_symbol("> ").highlight_style(
            Style::default()
                .fg(tailwind::ROSE.c500)
                .add_modifier(Modifier::BOLD),
        );

        StatefulWidget::render(list, inner_chunks[2], buf, &mut state.state);
    }
}

fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_line_len = 0;

    for word in text.split_whitespace() {
        if current_line_len + word.len() + 1 > width {
            result.push('\n');
            current_line_len = 0;
        } else if current_line_len > 0 {
            result.push(' ');
            current_line_len += 1;
        }
        result.push_str(word);
        current_line_len += word.len();
    }
    result
}

impl TaskColumnState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { state }
    }
}
