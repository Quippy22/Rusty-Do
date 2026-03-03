use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Buffer, Rect},
    style::{Modifier, Style},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
        Wrap,
    },
};

use crate::models::task::Task;
use crate::ui::theme::theme;

#[derive(Clone)]
pub struct TaskColumnState {
    pub state: ListState,
}

pub struct TaskColumn<'a> {
    pub task: &'a Task,
    pub is_focused: bool,
    pub index: usize,
    pub total_tasks: usize,
}

impl<'a> TaskColumn<'a> {
    pub fn new(task: &'a Task, is_focused: bool, index: usize, total_tasks: usize) -> Self {
        Self {
            task,
            is_focused,
            index,
            total_tasks,
        }
    }
}

impl<'a> StatefulWidget for TaskColumn<'a> {
    type State = TaskColumnState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_color = if self.is_focused {
            theme().border_focused
        } else {
            theme().border_unfocused
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
                Constraint::Max(5),    // Header (Name + Progress + Counter)
                Constraint::Length(1), // Separator line
                Constraint::Min(0),    // List of subtasks
            ])
            .split(inner_area);

        // -- The Header --
        let header = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Name
                Constraint::Min(0),    // Spacer
                Constraint::Length(1), // Completion
                Constraint::Length(1), // Task x of y
            ])
            .split(inner_chunks[0]);

        let title = Paragraph::new(self.task.name.to_string())
            .centered()
            .style(
                Style::default()
                    .fg(theme().title_main)
                    .add_modifier(Modifier::BOLD),
            )
            .wrap(Wrap { trim: true });
        title.render(header[0], buf);

        let completion = format!("Completion {}", self.task.completion);
        let completion_color = if self.task.is_done {
            theme().completion_done
        } else {
            theme().completion_pending
        };
        Paragraph::new(completion)
            .alignment(Alignment::Left)
            .style(Style::default().fg(completion_color))
            .render(header[2], buf);

        let counter_text = format!("Task {} of {}", self.index + 1, self.total_tasks);
        Paragraph::new(counter_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(theme().help_text))
            .render(header[3], buf);

        // -- The Separator --
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(border_color))
            .render(inner_chunks[1], buf);

        // -- The Body --
        let wrap_width = area.width.saturating_sub(4) as usize;
        let items: Vec<ListItem> = self
            .task
            .subtasks
            .iter()
            .map(|s| {
                let symbol = if s.is_done { "[x]" } else { "[ ]" };
                let full_text = format!("{} {}", symbol, s.name);

                // Wrap text to fit inside the column
                let wrapped = wrap_text(&full_text, wrap_width);
                ListItem::new(wrapped)
            })
            .collect();

        let list = List::new(items).highlight_symbol("> ").highlight_style(
            Style::default()
                .fg(theme().highlight)
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
