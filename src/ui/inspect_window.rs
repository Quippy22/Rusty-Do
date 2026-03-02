use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::palette::tailwind,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

#[derive(PartialEq, Clone)]
pub enum InspectMode {
    View,
    Edit,
    Add,
}

#[derive(PartialEq, Clone)]
pub enum InspectField {
    Title,
    Description,
    Contents,
}

pub enum InspectorAction {
    Submit,
    Cancel,
    Edit,
}

#[derive(PartialEq, Clone)]
pub struct Inspector {
    pub mode: InspectMode,
    pub title_input: String,
    pub desc_input: String,
    pub task_input: String, // Buffer for the task currently being typed
    pub list_items: Vec<String>,
    pub list_label: String,
    pub focused_field: InspectField,
    pub cursor_pos: usize,
}

use crate::models::notebook::Notebook;
use crate::models::task::Task;

impl Inspector {
    pub fn new(
        mode: InspectMode,
        title_input: String,
        desc_input: String,
        list_items: Vec<String>,
        list_label: String,
    ) -> Self {
        let cursor_pos = title_input.len();
        Self {
            mode,
            title_input,
            desc_input,
            task_input: String::new(),
            list_items,
            list_label,
            focused_field: InspectField::Title,
            cursor_pos,
        }
    }

    pub fn setup(nb: Option<&Notebook>, task: Option<&Task>, list_label: String) -> Self {
        let (mode, title, desc, items) = match (nb, task) {
            (Some(n), _) => (
                InspectMode::Edit,
                n.name.clone(),
                n.description.clone(),
                n.tasks.iter().map(|t| t.name.clone()).collect(),
            ),
            (_, Some(t)) => (
                InspectMode::Edit,
                t.name.clone(),
                t.description.clone(),
                t.subtasks.iter().map(|s| s.name.clone()).collect(),
            ),
            _ => (InspectMode::Add, String::new(), String::new(), Vec::new()),
        };

        let cursor_pos = title.len();
        Self {
            mode,
            title_input: title,
            desc_input: desc,
            task_input: String::new(),
            list_items: items,
            list_label,
            focused_field: InspectField::Title,
            cursor_pos,
        }
    }
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<InspectorAction> {
        if self.mode == InspectMode::View {
            match key.code {
                KeyCode::Esc => return Some(InspectorAction::Cancel),
                KeyCode::Char('e') => return Some(InspectorAction::Edit),
                _ => return None,
            }
        }

        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            // -- Submit --
            KeyCode::Char('s') if ctrl => {
                if !self.title_input.trim().is_empty() {
                    self.flush_task_buffer();
                    return Some(InspectorAction::Submit);
                }
            }
            KeyCode::Enter if alt => {
                if !self.title_input.trim().is_empty() {
                    self.flush_task_buffer();
                    return Some(InspectorAction::Submit);
                }
            }

            // -- Exit --
            KeyCode::Esc => return Some(InspectorAction::Cancel),

            // -- Navigation --
            KeyCode::Tab => {
                self.focused_field = match self.focused_field {
                    InspectField::Title => InspectField::Description,
                    InspectField::Description => InspectField::Contents,
                    InspectField::Contents => InspectField::Title,
                };
                self.reset_cursor_to_end();
            }
            KeyCode::BackTab => {
                self.focused_field = match self.focused_field {
                    InspectField::Title => InspectField::Contents,
                    InspectField::Description => InspectField::Title,
                    InspectField::Contents => InspectField::Description,
                };
                self.reset_cursor_to_end();
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                let len = self.get_current_buffer().len();
                if self.cursor_pos < len {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Up => {
                if self.focused_field == InspectField::Description {
                    self.move_cursor_vertical(-1);
                } else {
                    // Cycle fields upwards
                    self.focused_field = match self.focused_field {
                        InspectField::Title => InspectField::Contents,
                        InspectField::Description => InspectField::Title,
                        InspectField::Contents => InspectField::Description,
                    };
                    self.reset_cursor_to_end();
                }
            }
            KeyCode::Down => {
                if self.focused_field == InspectField::Description {
                    self.move_cursor_vertical(1);
                } else {
                    // Cycle fields downwards
                    self.focused_field = match self.focused_field {
                        InspectField::Title => InspectField::Description,
                        InspectField::Description => InspectField::Contents,
                        InspectField::Contents => InspectField::Title,
                    };
                    self.reset_cursor_to_end();
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.get_current_buffer().len();
            }

            // -- Guided Flow --
            KeyCode::Enter => match self.focused_field {
                InspectField::Title => {
                    if !self.title_input.trim().is_empty() {
                        self.focused_field = InspectField::Description;
                        self.reset_cursor_to_end();
                    }
                }
                InspectField::Description => {
                    if shift {
                        self.focused_field = InspectField::Contents;
                        self.reset_cursor_to_end();
                    } else {
                        self.desc_input.insert(self.cursor_pos, '\n');
                        self.cursor_pos += 1;
                    }
                }
                InspectField::Contents => {
                    self.flush_task_buffer();
                    self.cursor_pos = 0;
                }
            },

            // -- Typing --
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    match self.focused_field {
                        InspectField::Title => {
                            self.title_input.remove(self.cursor_pos - 1);
                        }
                        InspectField::Description => {
                            self.desc_input.remove(self.cursor_pos - 1);
                        }
                        InspectField::Contents => {
                            self.task_input.remove(self.cursor_pos - 1);
                        }
                    }
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Delete => {
                let len = self.get_current_buffer().len();
                if self.cursor_pos < len {
                    match self.focused_field {
                        InspectField::Title => {
                            self.title_input.remove(self.cursor_pos);
                        }
                        InspectField::Description => {
                            self.desc_input.remove(self.cursor_pos);
                        }
                        InspectField::Contents => {
                            self.task_input.remove(self.cursor_pos);
                        }
                    }
                }
            }

            KeyCode::Char(c) => {
                match self.focused_field {
                    InspectField::Title => {
                        self.title_input.insert(self.cursor_pos, c);
                    }
                    InspectField::Description => {
                        self.desc_input.insert(self.cursor_pos, c);
                    }
                    InspectField::Contents => {
                        self.task_input.insert(self.cursor_pos, c);
                    }
                }
                self.cursor_pos += 1;
            }

            _ => {}
        }

        None
    }

    fn reset_cursor_to_end(&mut self) {
        self.cursor_pos = self.get_current_buffer().len();
    }

    fn get_current_buffer(&self) -> &String {
        match self.focused_field {
            InspectField::Title => &self.title_input,
            InspectField::Description => &self.desc_input,
            InspectField::Contents => &self.task_input,
        }
    }

    fn move_cursor_vertical(&mut self, dir: i32) {
        let text = &self.desc_input;
        if text.is_empty() {
            return;
        }

        // 1. Find line boundaries
        let lines: Vec<&str> = text.split('\n').collect();
        // If it ends with \n, split leaves an empty string, which is correct

        // 2. Find current line and column
        let mut current_line = 0;
        let mut current_col = 0;
        let mut char_count = 0;

        for (i, line) in lines.iter().enumerate() {
            let next_boundary = char_count + line.len();
            if self.cursor_pos <= next_boundary {
                current_line = i;
                current_col = self.cursor_pos - char_count;
                break;
            }
            char_count += line.len() + 1; // +1 for the newline
        }

        // 3. Move to target line
        let target_line = (current_line as i32 + dir)
            .max(0)
            .min(lines.len() as i32 - 1) as usize;

        if target_line == current_line {
            return;
        }

        // 4. Calculate new cursor position
        let mut new_pos = 0;
        for i in 0..target_line {
            new_pos += lines[i].len() + 1;
        }

        // Clamp column to target line length
        let target_col = current_col.min(lines[target_line].len());
        new_pos += target_col;

        self.cursor_pos = new_pos;
    }

    fn flush_task_buffer(&mut self) {
        if !self.task_input.trim().is_empty() {
            self.list_items.push(self.task_input.clone());
            self.task_input.clear();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.title_input.trim().is_empty()
            && self.desc_input.trim().is_empty()
            && self.list_items.is_empty()
            && self.task_input.trim().is_empty()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Title (Compact)
                Constraint::Percentage(30), // Description
                Constraint::Fill(1),        // Contents (Rest of the space)
            ])
            .split(area);

        // Colors
        let focused_color = tailwind::ROSE.c500;
        let default_color = tailwind::WHITE;

        // -- Title --
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(match &self.focused_field {
                InspectField::Title => focused_color,
                _ => default_color,
            })
            .title("Title")
            .title_alignment(Alignment::Left);

        let title_block_inner = title_block.inner(chunks[0]);
        f.render_widget(title_block, chunks[0]);

        let title_display =
            if self.focused_field == InspectField::Title && self.mode != InspectMode::View {
                let prefix = &self.title_input[..self.cursor_pos];
                let suffix = &self.title_input[self.cursor_pos..];
                format!("{}▎{}", prefix, suffix)
            } else {
                self.title_input.clone()
            };
        f.render_widget(Paragraph::new(title_display), title_block_inner);

        // -- Description --
        let description_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(match &self.focused_field {
                InspectField::Description => focused_color,
                _ => default_color,
            })
            .title("Description")
            .title_alignment(Alignment::Left);

        let description_block_inner = description_block.inner(chunks[1]);
        f.render_widget(description_block, chunks[1]);

        let desc_display =
            if self.focused_field == InspectField::Description && self.mode != InspectMode::View {
                let prefix = &self.desc_input[..self.cursor_pos];
                let suffix = &self.desc_input[self.cursor_pos..];
                format!("{}▎{}", prefix, suffix)
            } else {
                self.desc_input.clone()
            };
        let desc_paragraph = Paragraph::new(desc_display).wrap(Wrap { trim: true });
        f.render_widget(desc_paragraph, description_block_inner);

        // -- Contents --
        let contents_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(match &self.focused_field {
                InspectField::Contents => focused_color,
                _ => default_color,
            })
            .title(self.list_label.as_str())
            .title_alignment(Alignment::Left);

        let contents_block_inner = contents_block.inner(chunks[2]);
        f.render_widget(contents_block, chunks[2]);

        let mut items: Vec<ListItem> = self
            .list_items
            .iter()
            .map(|name| ListItem::new(format!(" • {}", name)))
            .collect();

        // Show the current typing buffer if focused
        if self.focused_field == InspectField::Contents && self.mode != InspectMode::View {
            let prefix = &self.task_input[..self.cursor_pos];
            let suffix = &self.task_input[self.cursor_pos..];
            items.push(ListItem::new(format!(" • {}▎{}", prefix, suffix)));
        }

        f.render_widget(List::new(items), contents_block_inner);
    }
}
