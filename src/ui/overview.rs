use ratatui::{
    layout::{ Alignment, Constraint, Direction, Layout, Rect },
    widgets::{ Block, BorderType, Borders, List, ListState, ListItem, Paragraph },
    Frame,
};
use crossterm::event::{ KeyCode, KeyEvent };

use crate::models::notebook::Notebook;

#[derive(Clone)]
pub struct Overview {
    pub notebooks: Vec<Notebook>,
    pub state: ListState,
}

pub enum OverviewAction {
    DeleteNotebook,
    RenameNotebook, 
}

impl Overview {
    // Initialize
    pub fn new(notebooks: Vec<Notebook>) -> Self {
        let mut state = ListState::default();
        if !notebooks.is_empty() {
            state.select(Some(0));
        }
        Self { notebooks, state }
    }

    // The render logic
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // 1. Define the split
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(2)
            .split(area);

        // 2. Left block
        // List of 'notebooks'
        let notebooks_block = Block::default()
            .title("Notebooks")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let notebooks: Vec<ListItem> = self.notebooks
            .iter()
            .map(|n| ListItem::new(n.name.as_str()))
            .collect();

        let notebook_list = List::new(notebooks).highlight_symbol("> ").block(notebooks_block);
        f.render_stateful_widget(notebook_list, chunks[0], &mut self.state);

        // 3. Right block
        // Entries (preview)
        let preview_block = Block::default()
            .title("Preview")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let selected_idx = self.state.selected().unwrap_or(0);

        let display_text = if let Some(notebook) = self.notebooks.get(selected_idx) {
            if notebook.tasks.is_empty() {
                String::from("No tasks yet.")
            } else {
                notebook.tasks
                    .iter()
                    .map(|t| format!("• {}", t.name))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        } else {
            String::from("No notebook selected.")
        };

        let preview_text = Paragraph::new(display_text).block(preview_block);
        f.render_widget(preview_text, chunks[1]);
    }
}

impl Overview {
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<OverviewAction> {
        match key.code {
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
                None
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.state.select_last();
                None
            }
            KeyCode::Char('d')  => Some(OverviewAction::DeleteNotebook),
            KeyCode::Char('r') => Some(OverviewAction::RenameNotebook),
            _ => None,
        }
    }

    // Movement
    pub fn next(&mut self) {
        if self.notebooks.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.notebooks.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.notebooks.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 { self.notebooks.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
