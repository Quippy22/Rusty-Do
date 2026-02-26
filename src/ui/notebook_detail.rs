use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    widgets::{Block, BorderType, Borders},
};

use crate::models::notebook::Notebook;
use crate::ui::task_column::{TaskColumn, TaskColumnState};

#[derive(Clone)]
pub struct NotebookDetail {
    pub notebook: Option<Notebook>,
    pub task_states: Vec<TaskColumnState>,
    pub selected_task_idx: Option<usize>,
    pub scroll_offset: usize,
}

pub enum NotebookViewAction {
    Exit,
    RenameTask,
    RenameSubtask,
    DeleteTask,
    DeleteSubtask,
    AddTaskBefore,
    AddTaskAfter,
    EditTask,
    InspectTask,
}

impl NotebookDetail {
    pub fn new(nb: Option<Notebook>) -> Self {
        if let Some(notebook) = nb {
            let task_count = notebook.tasks.len();
            Self {
                notebook: Some(notebook),
                task_states: (0..task_count).map(|_| TaskColumnState::new()).collect(),
                selected_task_idx: if task_count > 0 { Some(0) } else { None },
                scroll_offset: 0,
            }
        } else {
            Self {
                notebook: None,
                task_states: Vec::new(),
                selected_task_idx: None,
                scroll_offset: 0,
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // 1. Get notebook data and calculate UI state first (to avoid borrow conflicts)
        let (notebook_name, column_widths) = match &self.notebook {
            Some(nb) => (nb.name.clone(), self.get_column_widths()),
            None => return,
        };

        // 2. Draw the outer window
        let window = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(notebook_name.as_str())
            .title_alignment(Alignment::Center);

        let inner_window = window.inner(area);
        f.render_widget(window, area);

        let vertical_margins = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10), // Top margin
                Constraint::Percentage(80), // Middle content area
                Constraint::Percentage(10), // Bottom margin
            ])
            .split(inner_window);

        let content_area = vertical_margins[1];

        // 3. Update scrolling state (needs &mut self)
        self.update_scroll_offset(&column_widths, content_area.width);
        let (visible_count, constraints) =
            self.get_visible_columns(&column_widths, content_area.width);

        if visible_count == 0 {
            return;
        }

        // 4. Horizontal layout
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .flex(Flex::Start)
            .split(content_area);

        // 5. Final Render (borrowing notebook immutably ONLY here)
        if let Some(notebook) = &self.notebook {
            for i in 0..visible_count {
                let task_idx = self.scroll_offset + i;
                if let Some(task) = notebook.tasks.get(task_idx) {
                    let is_focused = Some(task_idx) == self.selected_task_idx;
                    let widget = TaskColumn::new(task, is_focused);
                    f.render_stateful_widget(
                        widget,
                        horizontal_chunks[i],
                        &mut self.task_states[task_idx],
                    );
                }
            }
        }
    }
}

impl NotebookDetail {
    // -- Input Handling --
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<NotebookViewAction> {
        let task_count = if let Some(nb) = &self.notebook {
            nb.tasks.len()
        } else {
            0
        };

        match key.code {
            KeyCode::Esc => Some(NotebookViewAction::Exit),

            // Horizontal Navigation
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(selected) = self.selected_task_idx {
                    if selected > 0 {
                        self.selected_task_idx = Some(selected - 1);
                    } else if task_count > 0 {
                        self.selected_task_idx = Some(task_count - 1);
                    }
                }
                None
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if let Some(selected) = self.selected_task_idx {
                    if selected < task_count - 1 {
                        self.selected_task_idx = Some(selected + 1);
                    } else if task_count > 0 {
                        self.selected_task_idx = Some(0);
                    }
                }
                None
            }

            // Vertical Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.next_subtask();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous_subtask();
                None
            }

            // Toggles
            KeyCode::Char('x') => {
                if let Some(selected) = self.selected_task_idx {
                    let subtask_idx = self.task_states[selected].state.selected();
                    if let (Some(nb), Some(s_idx)) = (&mut self.notebook, subtask_idx) {
                        nb.tasks[selected].toggle_subtask(s_idx);
                    }
                }
                None
            }
            KeyCode::Char('X') => {
                if let Some(selected) = self.selected_task_idx {
                    if let Some(nb) = &mut self.notebook {
                        nb.tasks[selected].toggle_task();
                    }
                }
                None
            }

            // Renames
            KeyCode::Char('r') => Some(NotebookViewAction::RenameTask),
            KeyCode::Char('e') => Some(NotebookViewAction::RenameSubtask),
            KeyCode::Char('E') => Some(NotebookViewAction::EditTask),

            // Adds
            KeyCode::Char('a') => Some(NotebookViewAction::AddTaskAfter),
            KeyCode::Char('i') => Some(NotebookViewAction::AddTaskBefore),

            // Deletes
            KeyCode::Char('D') => Some(NotebookViewAction::DeleteTask),
            KeyCode::Char('d') => Some(NotebookViewAction::DeleteSubtask),

            KeyCode::Enter => Some(NotebookViewAction::InspectTask),

            _ => None,
        }
    }

    fn next_subtask(&mut self) {
        if let Some(selected) = self.selected_task_idx {
            let subtask_count = self
                .notebook
                .as_ref()
                .map(|n| n.tasks[selected].subtasks.len())
                .unwrap_or(0);
            if subtask_count > 0 {
                let state = &mut self.task_states[selected].state;
                let i = match state.selected() {
                    Some(i) => {
                        if i >= subtask_count - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.select(Some(i));
            }
        }
    }

    fn previous_subtask(&mut self) {
        if let Some(selected) = self.selected_task_idx {
            let subtask_count = self
                .notebook
                .as_ref()
                .map(|n| n.tasks[selected].subtasks.len())
                .unwrap_or(0);
            if subtask_count > 0 {
                let state = &mut self.task_states[selected].state;
                let i = match state.selected() {
                    Some(i) => {
                        if i == 0 {
                            subtask_count - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.select(Some(i));
            }
        }
    }
}

impl NotebookDetail {
    // -- Helpers --
    fn update_scroll_offset(&mut self, widths: &[u16], area_width: u16) {
        let Some(selected) = self.selected_task_idx else {
            return;
        };

        if selected < self.scroll_offset {
            self.scroll_offset = selected;
        } else {
            // Check if selected is within visible range
            let mut current_w = 0;
            let mut in_view = false;
            for (idx, &w) in widths.iter().enumerate().skip(self.scroll_offset) {
                if current_w + w > area_width {
                    break;
                }
                if idx == selected {
                    in_view = true;
                    break;
                }
                current_w += w;
            }

            if !in_view {
                // Slide scroll_offset right until 'selected' is visible
                let mut new_offset = self.scroll_offset;
                loop {
                    let mut view_w = 0;
                    let mut visible = false;
                    for (i, &w) in widths.iter().enumerate().skip(new_offset) {
                        if view_w + w > area_width {
                            break;
                        }
                        if i == selected {
                            visible = true;
                            break;
                        }
                        view_w += w;
                    }
                    if visible || new_offset >= selected {
                        break;
                    }
                    new_offset += 1;
                }
                self.scroll_offset = new_offset;
            }
        }
    }

    fn get_visible_columns(&self, widths: &[u16], area_width: u16) -> (usize, Vec<Constraint>) {
        let mut count = 0;
        let mut total_w = 0;
        let mut constraints = Vec::new();

        for &w in widths.iter().skip(self.scroll_offset) {
            if total_w + w > area_width {
                break;
            }
            total_w += w;
            count += 1;
            constraints.push(Constraint::Length(w));
        }

        if count == 0 && !widths.is_empty() {
            // Force at least one if it doesn't fit
            let w = widths[self.scroll_offset];
            count = 1;
            constraints.push(Constraint::Length(w));
        }

        (count, constraints)
    }

    fn get_column_widths(&self) -> Vec<u16> {
        if let Some(notebook) = &self.notebook {
            notebook
                .tasks
                .iter()
                .map(|t| {
                    let mut max_w = t.name.len();
                    for sub in &t.subtasks {
                        let w = sub.name.len() + 6; // "[ ] " icon and "> "
                        if w > max_w {
                            max_w = w;
                        }
                    }
                    (max_w.clamp(20, 70) as u16) + 2 // +2 for borders
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
