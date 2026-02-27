pub mod actions;
pub mod input;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app::actions::PendingAction;
use crate::models::notebook::Notebook;
use crate::storage::{paths::FileSystem, persistence::Persistence};
use crate::ui::{
    confirm::ConfirmPopup, inspect_window::Inspector, notebook_detail::NotebookDetail,
    overview::Overview, rename::RenamePopup,
};

#[derive(Clone)]
pub enum AppMode {
    // -- Windows --
    Overview,       // The main window
    NotebookDetail, // See and interact with the tasks of one notebook
    TaskEditor,     // Add/edit a task
    Add(PendingAction),

    // -- Popups --
    Confirm(ConfirmPopup, PendingAction),
    Rename(RenamePopup, PendingAction),
    Help, // See keybinds
}

impl AppMode {
    pub fn can_quit(&self) -> bool {
        match self {
            AppMode::Overview => true,
            AppMode::NotebookDetail => true,
            _ => false,
        }
    }

    pub fn is_popup(&self) -> bool {
        match self {
            AppMode::Rename(_, _) => true,
            AppMode::Confirm(_, _) => true,
            AppMode::Help => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct App {
    // General
    pub mode: AppMode,
    pub last_window: AppMode,
    pub storage: Persistence,
    pub should_quit: bool,
    pub notebooks: Vec<Notebook>,
    pub selected_notebook_idx: usize,
    // Overview
    pub overview: Overview,
    // Notebook Detail
    pub nb_detail: NotebookDetail,
    // Inspector (Add/Edit Mode)
    pub inspector: Inspector,
}

impl App {
    pub fn new(filesystem: FileSystem) -> Self {
        let storage = Persistence::new(filesystem);
        let index = storage.validate_and_sync_index().unwrap_or_default();
        let mut notebooks: Vec<Notebook> = Vec::new();
        for meta in index.notebooks {
            if let Ok(nb) = storage.load_notebook(&meta.id) {
                notebooks.push(nb);
            }
        }

        Self {
            mode: AppMode::Overview,
            last_window: AppMode::Overview,
            should_quit: false,
            selected_notebook_idx: 0,
            storage,
            overview: Overview::new(notebooks.clone()),
            notebooks,
            nb_detail: NotebookDetail::new(None),
            inspector: Inspector::setup(None, None, String::from("Tasks")),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match &mut self.mode {
            // -- Normal Windows --
            AppMode::Overview => self.overview.render(f, area, true),
            AppMode::NotebookDetail => self.nb_detail.render(f, area),

            // -- Popups --
            m if m.is_popup() => {
                match &self.last_window {
                    AppMode::Overview => self.overview.render(f, area, true),
                    AppMode::NotebookDetail => self.nb_detail.render(f, area),
                    _ => {}
                }
                match &mut self.mode {
                    AppMode::Confirm(popup, _) => popup.render(f, area),
                    AppMode::Rename(popup, _) => popup.render(f, area),
                    _ => {}
                }
            }

            // -- Inspector --
            AppMode::Add(_) => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .spacing(1)
                    .split(area);

                self.inspector.render(f, chunks[1]);

                match &self.last_window {
                    AppMode::Overview => self.overview.render(f, chunks[0], false),
                    AppMode::NotebookDetail => self.nb_detail.render(f, chunks[0]),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        input::handle_input(self, key);
    }

    pub fn refresh_notebooks_list(&mut self) {
        let index = self.storage.validate_and_sync_index().unwrap_or_default();
        let mut notebooks = Vec::new();
        let current_id = self
            .overview
            .state
            .selected()
            .and_then(|idx| self.notebooks.get(idx))
            .map(|nb| nb.id.clone());

        for meta in index.notebooks {
            if let Ok(nb) = self.storage.load_notebook(&meta.id) {
                notebooks.push(nb);
            }
        }
        self.notebooks = notebooks;
        self.overview.notebooks = self.notebooks.clone();

        if let Some(id) = current_id {
            if let Some(new_idx) = self.notebooks.iter().position(|nb| nb.id == id) {
                self.overview.state.select(Some(new_idx));
                self.selected_notebook_idx = new_idx;
            }
        }

        if let Some(nb) = self.notebooks.get(self.selected_notebook_idx) {
            self.nb_detail.notebook = Some(nb.clone());
        }

        self.overview.sync_inspector();
    }

    pub fn refresh_nb_detail(&mut self, notebook: Notebook) {
        // 1. Update master list
        self.notebooks[self.selected_notebook_idx] = notebook.clone();

        // 2. Update active view
        self.nb_detail.notebook = Some(notebook.clone());

        // 3. Sync UI states (re-initialize to match new counts)
        self.nb_detail.task_states = (0..notebook.tasks.len())
            .map(|_| crate::ui::task_column::TaskColumnState::new())
            .collect();

        // 4. Persist
        let _ = self.storage.save_notebook(&notebook);
    }
}
