use crossterm::event::{ KeyEvent, KeyCode };
use ratatui::Frame;

use crate::models::notebook::Notebook;
use crate::ui::overview::{ Overview, OverviewAction };

#[derive(Clone)]
pub enum AppMode {
    Overview, // The main window
    NotebookDetail, // See and interact with the tasks of one notebook
    TaskEditor, // Add/edit a task
    Help, // See keybinds
}

#[derive(Clone)]
pub struct App {
    // General
    pub mode: AppMode,
    pub should_quit: bool,
    pub selected_notebook_idx: usize,
    // Overview
    pub overview: Overview,
    pub notebooks: Vec<Notebook>,
}

impl App {
    pub fn new(notebooks: Vec<Notebook>) -> Self {
        let overview = Overview::new(notebooks.clone());

        Self {
            mode: AppMode::Overview,
            should_quit: false,
            selected_notebook_idx: 0,
            overview,
            notebooks,
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match self.mode {
            AppMode::Overview => {
                self.overview.render(f, area);
            }

            _ => {}
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl App {
    // -- Key Handeling --
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global keys
        if key.code == KeyCode::Char('q') && !matches!(self.mode, AppMode::TaskEditor) {
            self.quit();
            return;
        }

        // Mode-specific keys
        match self.mode {
            AppMode::Overview => self.overview_handle_key(key),
            _ => {}
        }
    }

    pub fn overview_handle_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Enter {
            // Save the index before we leave the Overview mode
            if let Some(idx) = self.overview.state.selected() {
                self.selected_notebook_idx = idx;
            }

            self.mode = AppMode::NotebookDetail;
            return;
        }

        if let Some(action) = self.overview.handle_key(key) {
            match action {
                OverviewAction::DeleteNotebook => {
                    self.delete_selected_notebook();
                }
                OverviewAction::RenameNotebook => {}
            }
        }
    }

    pub fn delete_selected_notebook(&mut self) {
        if let Some(idx) = self.overview.state.selected() {
            // Remove the notebook
            self.notebooks.remove(idx);
            // Sync the data
            self.overview.notebooks = self.notebooks.clone();

            // Adjust selection if needed
            if self.overview.notebooks.is_empty() {
                self.overview.state.select(None);
            } else if idx >= self.overview.notebooks.len() {
                self.overview.state.select(Some(self.overview.notebooks.len() - 1));
            }
        }
    }
}
