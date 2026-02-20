use crossterm::event::{ KeyEvent, KeyCode };
use ratatui::Frame;

use crate::models::notebook::Notebook;
use crate::ui::overview::Overview;

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

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global keys
        if key.code == KeyCode::Char('q') && !matches!(self.mode, AppMode::TaskEditor) {
            self.quit();
            return;
        }

        // Mode-specific keys
        match self.mode {
            AppMode::Overview => self.overview.handle_key(key),
            _ => {}
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
