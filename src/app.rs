use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

use crate::models::notebook::Notebook;
use crate::ui::{
    confirm::ConfirmPopup,
    overview::{Overview, OverviewAction},
};

#[derive(Clone)]
pub enum AppMode {
    // -- Windows --
    Overview,       // The main window
    NotebookDetail, // See and interact with the tasks of one notebook
    TaskEditor,     // Add/edit a task

    // -- Popups --
    Confirm(ConfirmPopup, PendingAction),
    Help, // See keybinds
}

#[derive(Clone)]
pub enum PendingAction {
    DeleteNotebook,
}

#[derive(Clone)]
pub struct App {
    // General
    pub mode: AppMode,
    pub should_quit: bool,
    pub selected_notebook_idx: usize,
    pub confirm: Option<ConfirmPopup>,
    // Overview
    pub overview: Overview,
    pub notebooks: Vec<Notebook>,
}

impl App {
    pub fn new(notebooks: Vec<Notebook>) -> Self {
        Self {
            mode: AppMode::Overview,
            should_quit: false,
            selected_notebook_idx: 0,
            overview: Overview::new(notebooks.clone()),
            notebooks,
            confirm: None,
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match &self.mode {
            AppMode::Overview | AppMode::Confirm(_, _) => {
                self.overview.render(f, area);
            }

            _ => {}
        }

        // Render ontop
        if let AppMode::Confirm(popup, _) = &self.mode {
            popup.render(f, area);
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl App {
    // -- Input Handeling --
    pub fn handle_input(&mut self, key: KeyEvent) {
        // Global keys
        if key.code == KeyCode::Char('q') && self.mode.can_quit() {
            self.quit();
            return;
        }

        // Mode-specific keys
        let current_mode = self.mode.clone();
        match current_mode {
            AppMode::Overview => self.overview_handle_input(key),
            AppMode::Confirm(popup, action) => {
                if let Some(confirmed) = popup.handle_input(key) {
                    if confirmed {
                        match action {
                            PendingAction::DeleteNotebook => self.delete_selected_notebook(),
                        }
                    }
                    // Reset the mode
                    self.mode = AppMode::Overview;
                }
            }
            _ => {}
        }
    }

    pub fn overview_handle_input(&mut self, key: KeyEvent) {
        if let Some(action) = self.overview.handle_input(key) {
            match action {
                OverviewAction::DeleteNotebook => {
                    // Create the popup
                    let popup = ConfirmPopup::new(
                        String::from("Delete notebook"),
                        String::from(format!("Are you sure you want to delete {}?", {
                            if let Some(noteb) = self.overview.state.selected() {
                                self.notebooks[noteb].name.clone()
                            } else {
                                String::from("")
                            }
                        })),
                    );
                    // Switch mode
                    self.mode = AppMode::Confirm(popup, PendingAction::DeleteNotebook);
                }
                OverviewAction::AccessNotebook => {
                    // Save the index before we leave the Overview mode
                    if let Some(idx) = self.overview.state.selected() {
                        self.selected_notebook_idx = idx;
                    }

                    self.mode = AppMode::NotebookDetail;
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
                self.overview
                    .state
                    .select(Some(self.overview.notebooks.len() - 1));
            }
        }
    }
}

impl AppMode {
    pub fn can_quit(&self) -> bool {
        match self {
            AppMode::TaskEditor => false,
            AppMode::Confirm(_, _) => false,

            _ => true,
        }
    }
}
