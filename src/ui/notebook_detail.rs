use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
                Constraint::Percentage(5),  // Top margin
                Constraint::Percentage(90), // Middle content area
                Constraint::Percentage(5),  // Bottom margin
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

        constraints.push(Constraint::Min(0));
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
