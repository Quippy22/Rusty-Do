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
        Self {
            mode,
            title_input,
            desc_input,
            task_input: String::new(),
            list_items,
            list_label,
            focused_field: InspectField::Title,
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

        Self {
            mode,
            title_input: title,
            desc_input: desc,
            task_input: String::new(),
            list_items: items,
            list_label,
            focused_field: InspectField::Title,
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

        match key.code {
            // -- Submit --
            KeyCode::Enter if alt => {
                if !self.title_input.trim().is_empty() {
                    // Flush the task buffer before submitting
                    if !self.task_input.trim().is_empty() {
                        self.list_items.push(self.task_input.clone());
                        self.task_input.clear();
                    }
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
            }
            KeyCode::BackTab => {
                self.focused_field = match self.focused_field {
                    InspectField::Title => InspectField::Contents,
                    InspectField::Description => InspectField::Title,
                    InspectField::Contents => InspectField::Description,
                };
            }

            // -- Guided Flow --
            KeyCode::Enter => match self.focused_field {
                InspectField::Title => {
                    if !self.title_input.trim().is_empty() {
                        self.focused_field = InspectField::Description;
                    }
                }
                InspectField::Description => {
                    if shift {
                        self.focused_field = InspectField::Contents;
                    } else {
                        self.desc_input.push('\n');
                    }
                }
                InspectField::Contents => {
                    if !self.task_input.trim().is_empty() {
                        self.list_items.push(self.task_input.clone());
                        self.task_input.clear();
                    }
                }
            },

            // -- Typing --
            KeyCode::Backspace => match self.focused_field {
                InspectField::Title => {
                    self.title_input.pop();
                }
                InspectField::Description => {
                    self.desc_input.pop();
                }
                InspectField::Contents => {
                    self.task_input.pop();
                }
            },

            KeyCode::Char(c) => match self.focused_field {
                InspectField::Title => {
                    self.title_input.push(c);
                }
                InspectField::Description => {
                    self.desc_input.push(c);
                }
                InspectField::Contents => {
                    self.task_input.push(c);
                }
            },

            _ => {}
        }

        None
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
                format!("{}█", self.title_input)
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
                format!("{}█", self.desc_input)
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
            items.push(ListItem::new(format!(" • {}█", self.task_input)));
        }

        f.render_widget(List::new(items), contents_block_inner);
    }
}
