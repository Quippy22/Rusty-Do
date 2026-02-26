use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::models::notebook::Notebook;
use crate::ui::inspect_window::{InspectMode, Inspector};

#[derive(Clone)]
pub struct Overview {
    pub notebooks: Vec<Notebook>,
    pub state: ListState,
    pub inspector: Inspector,
}

pub enum OverviewAction {
    DeleteNotebook,
    RenameNotebook,
    AccessNotebook,
    AddNotebook,
    EditNotebook,
}

impl Overview {
    // Initialize
    pub fn new(notebooks: Vec<Notebook>) -> Self {
        let mut state = ListState::default();
        if !notebooks.is_empty() {
            state.select(Some(0));
        }

        let inspector = Inspector::new(
            InspectMode::View,
            String::from("No Selection"),
            String::from("Select a notebook to see its details."),
            Vec::new(),
            String::from("Tasks"),
        );

        let mut overview = Self {
            notebooks,
            state,
            inspector,
        };

        overview.sync_inspector();
        overview
    }

    // The render logic
    pub fn render(&mut self, f: &mut Frame, area: Rect, show_detail: bool) {
        let (list_area, detail_area) = if show_detail {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .spacing(1)
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        // 1. Render List
        let notebooks_block = Block::default()
            .title("Notebooks")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let notebooks: Vec<ListItem> = self
            .notebooks
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let line = format!("{} {}", index + 1, name.name);
                ListItem::new(line)
            })
            .collect();

        let notebook_list = List::new(notebooks)
            .highlight_symbol("> ")
            .highlight_style(style::palette::tailwind::ROSE.c500)
            .block(notebooks_block);
        f.render_stateful_widget(notebook_list, list_area, &mut self.state);

        // 2. Render Details
        if let Some(da) = detail_area {
            self.render_detail(f, da);
        }
    }

    pub fn render_detail(&mut self, f: &mut Frame, area: Rect) {
        if self.notebooks.is_empty() {
            let empty_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Details")
                .title_alignment(Alignment::Left);
            f.render_widget(
                Paragraph::new("No notebooks found.").block(empty_block),
                area,
            );
        } else {
            self.inspector.render(f, area);
        }
    }

    pub fn sync_inspector(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(notebook) = self.notebooks.get(idx) {
                self.inspector.title_input = notebook.name.clone();
                self.inspector.desc_input = notebook.description.clone();
                self.inspector.list_items = notebook.tasks.iter().map(|t| t.name.clone()).collect();
            }
        }
    }
}

impl Overview {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<OverviewAction> {
        let action = match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
                None
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.state.select_first();
                self.sync_inspector();
                None
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.state.select_last();
                self.sync_inspector();
                None
            }
            KeyCode::Char('d') => Some(OverviewAction::DeleteNotebook),
            KeyCode::Char('r') => Some(OverviewAction::RenameNotebook),
            KeyCode::Char('a') => Some(OverviewAction::AddNotebook),
            KeyCode::Char('e') => Some(OverviewAction::EditNotebook),
            KeyCode::Enter => Some(OverviewAction::AccessNotebook),
            KeyCode::Char(c @ '1'..='5') => {
                let index: usize = (c.to_digit(10).unwrap_or(0) - 1) as usize;
                if self.notebooks.len() > index {
                    self.state.select(Some(index));
                    self.sync_inspector();
                }
                Some(OverviewAction::AccessNotebook)
            }

            _ => None,
        };
        action
    }

    // Movement
    pub fn next(&mut self) {
        if self.notebooks.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.notebooks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.sync_inspector();
    }

    pub fn previous(&mut self) {
        if self.notebooks.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.notebooks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.sync_inspector();
    }
}
