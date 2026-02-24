use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

use crate::models::notebook::Notebook;
use crate::storage::{paths::FileSystem, persistence::Persistence};
use crate::ui::notebook_detail::NotebookViewAction;
use crate::ui::{
    confirm::ConfirmPopup,
    notebook_detail::NotebookDetail,
    overview::{Overview, OverviewAction},
    rename::RenamePopup,
};

#[derive(Clone)]
pub enum AppMode {
    // -- Windows --
    Overview,       // The main window
    NotebookDetail, // See and interact with the tasks of one notebook
    TaskEditor,     // Add/edit a task

    // -- Popups --
    Confirm(ConfirmPopup, PendingAction),
    Rename(RenamePopup, PendingAction),
    Help, // See keybinds
}

#[derive(Clone)]
pub enum PendingAction {
    DeleteNotebook,
    DeleteTask,
    DeleteSubtask,
    RenameNotebook,
    RenameTask,
    RenameSubtask,
}

#[derive(Clone)]
pub struct App {
    // General
    pub mode: AppMode,
    pub last_window: AppMode,
    pub storage: Persistence,
    pub should_quit: bool,
    pub selected_notebook_idx: usize,
    // Overview
    pub overview: Overview,
    pub notebooks: Vec<Notebook>,
    // Notebook Detail
    pub nb_detail: NotebookDetail,
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
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match &self.mode {
            AppMode::Overview => self.overview.render(f, area),
            AppMode::NotebookDetail => self.nb_detail.render(f, area),

            m if m.is_popup() => {
                // Render the window in the background
                match &self.last_window {
                    AppMode::Overview => self.overview.render(f, area),
                    AppMode::NotebookDetail => self.nb_detail.render(f, area),

                    _ => {}
                }
                // Render the popup ontop
                match &self.mode {
                    AppMode::Confirm(popup, _) => popup.render(f, area),
                    AppMode::Rename(popup, _) => popup.render(f, area),

                    _ => {}
                }
            }

            _ => {}
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

        // Mode-specific keys (pattern matching to avoid cloning the entire mode)
        match &self.mode {
            AppMode::Overview => self.overview_handle_input(key),
            AppMode::NotebookDetail => self.notebook_detail_handle_input(key),
            AppMode::Rename(popup, action) => {
                let (p, a) = (popup.clone(), action.clone());
                self.handle_rename(p, a, key);
            }
            AppMode::Confirm(popup, action) => {
                let (p, a) = (popup.clone(), action.clone());
                if let Some(confirmed) = p.handle_input(key) {
                    if confirmed {
                        match a {
                            PendingAction::DeleteNotebook => self.delete_selected_notebook(),
                            PendingAction::DeleteTask => self.delete_selected_task(),
                            PendingAction::DeleteSubtask => self.delete_selected_subtask(),

                            _ => {}
                        }
                    }
                    self.mode = self.last_window.clone();
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
                        self.nb_detail = NotebookDetail::new(Some(self.notebooks[idx].clone()));
                        self.mode = AppMode::NotebookDetail;
                        self.last_window = AppMode::NotebookDetail;
                    }

                    self.mode = AppMode::NotebookDetail;
                }
                OverviewAction::RenameNotebook => {
                    if let Some(idx) = self.overview.state.selected() {
                        let current_name = self.notebooks[idx].name.clone();
                        let popup =
                            RenamePopup::new(String::from("Rename notebook"), current_name, None);
                        self.mode = AppMode::Rename(popup, PendingAction::RenameNotebook);
                    }
                }
            }
        }
    }

    pub fn notebook_detail_handle_input(&mut self, key: KeyEvent) {
        if let Some(action) = self.nb_detail.handle_input(key) {
            match action {
                NotebookViewAction::Exit => {
                    // Save before exiting
                    if let Some(nb) = &self.nb_detail.notebook {
                        let _ = self.storage.save_notebook(nb);
                    }
                    self.mode = AppMode::Overview;
                    self.overview = Overview::new(self.notebooks.clone());
                    self.last_window = AppMode::Overview;
                }
                NotebookViewAction::RenameTask => {
                    if let Some(nb) = &self.nb_detail.notebook {
                        if let Some(idx) = self.nb_detail.selected_task_idx {
                            let current_name = nb.tasks[idx].name.clone();
                            let popup =
                                RenamePopup::new(String::from("Rename task"), current_name, None);
                            self.mode = AppMode::Rename(popup, PendingAction::RenameTask);
                        }
                    }
                }
                NotebookViewAction::RenameSubtask => {
                    if let Some(nb) = &self.nb_detail.notebook {
                        if let Some(t_idx) = self.nb_detail.selected_task_idx {
                            if let Some(s_idx) = self.nb_detail.task_states[t_idx].state.selected()
                            {
                                let current_name = nb.tasks[t_idx].subtasks[s_idx].name.clone();
                                let popup = RenamePopup::new(
                                    String::from("Rename subtask"),
                                    current_name,
                                    None,
                                );
                                self.mode = AppMode::Rename(popup, PendingAction::RenameSubtask);
                            }
                        }
                    }
                }
                NotebookViewAction::DeleteTask => {
                    if let Some(nb) = &self.nb_detail.notebook {
                        if let Some(idx) = self.nb_detail.selected_task_idx {
                            let current_name = nb.tasks[idx].name.clone();
                            let popup = ConfirmPopup::new(
                                String::from("Delete task"),
                                String::from(format!(
                                    "Are you sure you want to delete {}?",
                                    current_name
                                )),
                            );
                            self.mode = AppMode::Confirm(popup, PendingAction::DeleteTask);
                        }
                    }
                }
                NotebookViewAction::DeleteSubtask => {
                    if let Some(nb) = &self.nb_detail.notebook {
                        if let Some(t_idx) = self.nb_detail.selected_task_idx {
                            if let Some(s_idx) = self.nb_detail.task_states[t_idx].state.selected()
                            {
                                let current_name = nb.tasks[t_idx].subtasks[s_idx].name.clone();
                                let popup = ConfirmPopup::new(
                                    String::from("Delete subtask"),
                                    String::from(format!(
                                        "Are you sure you want to delete {}?",
                                        current_name
                                    )),
                                );
                                self.mode = AppMode::Confirm(popup, PendingAction::DeleteSubtask);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn handle_rename(&mut self, mut popup: RenamePopup, action: PendingAction, key: KeyEvent) {
        match popup.handle_input(key) {
            Some(true) => {
                match action {
                    PendingAction::RenameNotebook => {
                        // Check for duplicate names
                        let name_exists = self.notebooks.iter().any(|n| n.name == popup.input);
                        if name_exists {
                            popup.warning = "Name is already used.".to_string();
                            self.mode = AppMode::Rename(popup, action);
                            return;
                        }
                        // Apply the change
                        let new_name = popup.input.clone();
                        self.apply_rename(action, new_name);
                    }
                    _ => {
                        let new_name = popup.input.clone();
                        self.apply_rename(action, new_name);
                    }
                }
            }
            Some(false) => {
                self.mode = self.last_window.clone();
            }
            None => {
                // Keep the mode updated with the current popup state (typing)
                self.mode = AppMode::Rename(popup, action);
            }
        }
    }

    pub fn apply_rename(&mut self, action: PendingAction, new_name: String) {
        match action {
            PendingAction::RenameNotebook => {
                if let Some(idx) = self.overview.state.selected() {
                    self.notebooks[idx].name = new_name;
                    // Save renamed notebook
                    let _ = self.storage.save_notebook(&self.notebooks[idx]);
                    self.overview.notebooks = self.notebooks.clone();
                }
            }
            PendingAction::RenameTask => {
                if let Some(notebook) = &mut self.nb_detail.notebook {
                    if let Some(idx) = self.nb_detail.selected_task_idx {
                        notebook.tasks[idx].name = new_name;
                        // Save modified notebook
                        let _ = self.storage.save_notebook(notebook);
                    }
                }
            }
            PendingAction::RenameSubtask => {
                if let Some(notebook) = &mut self.nb_detail.notebook {
                    if let Some(t_idx) = self.nb_detail.selected_task_idx {
                        if let Some(s_idx) = self.nb_detail.task_states[t_idx].state.selected() {
                            notebook.tasks[t_idx].subtasks[s_idx].name = new_name;
                            // Save modified notebook
                            let _ = self.storage.save_notebook(notebook);
                        }
                    }
                }
            }
            _ => {}
        }
        self.mode = self.last_window.clone();
    }

    pub fn delete_selected_notebook(&mut self) {
        if let Some(idx) = self.overview.state.selected() {
            let notebook_id = self.notebooks[idx].id.clone();
            // Remove the file from disk
            let path = self.storage.fs.get_notebook_path(&notebook_id);
            let _ = std::fs::remove_file(path);

            // Remove the notebook from memory
            self.notebooks.remove(idx);

            // Sync the index
            let _ = self.storage.validate_and_sync_index();

            // Sync the UI
            self.overview = Overview::new(self.notebooks.clone());

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

    pub fn delete_selected_task(&mut self) {
        if let Some(nb) = &mut self.nb_detail.notebook {
            if let Some(idx) = self.nb_detail.selected_task_idx {
                // 1. Remove the task data
                nb.tasks.remove(idx);

                // 2. Remove the UI state for that task
                if idx < self.nb_detail.task_states.len() {
                    self.nb_detail.task_states.remove(idx);
                }

                // 3. Adjust selection
                if nb.tasks.is_empty() {
                    self.nb_detail.selected_task_idx = None;
                } else if idx >= nb.tasks.len() {
                    self.nb_detail.selected_task_idx = Some(nb.tasks.len() - 1);
                }
            }
            // 4. Save to disk
            let _ = self.storage.save_notebook(nb);
            // 5. Sync to master list
            self.notebooks[self.selected_notebook_idx] = nb.clone();
        }
    }

    pub fn delete_selected_subtask(&mut self) {
        if let Some(nb) = &mut self.nb_detail.notebook {
            if let Some(t_idx) = self.nb_detail.selected_task_idx {
                if let Some(s_idx) = self.nb_detail.task_states[t_idx].state.selected() {
                    // 1. Remove the subtask data
                    nb.tasks[t_idx].subtasks.remove(s_idx);

                    // 2. Recalculate task completion
                    nb.tasks[t_idx].recalculate_completion();

                    // 3. Adjust selection
                    let subtask_count = nb.tasks[t_idx].subtasks.len();
                    if subtask_count == 0 {
                        self.nb_detail.task_states[t_idx].state.select(None);
                    } else if s_idx >= subtask_count {
                        self.nb_detail.task_states[t_idx]
                            .state
                            .select(Some(subtask_count - 1));
                    }
                }
            }
            // 4. Save to disk
            let _ = self.storage.save_notebook(nb);
            // 5. Sync to master list
            self.notebooks[self.selected_notebook_idx] = nb.clone();
        }
    }
}

impl AppMode {
    pub fn can_quit(&self) -> bool {
        match self {
            AppMode::TaskEditor => false,
            AppMode::Confirm(_, _) => false,
            AppMode::Rename(_, _) => false,

            _ => true,
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
