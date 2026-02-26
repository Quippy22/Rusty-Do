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

use crate::ui::inspect_window::{Inspector, InspectorAction};

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

#[derive(Clone)]
pub enum PendingAction {
    DeleteNotebook,
    DeleteTask,
    DeleteSubtask,
    RenameNotebook,
    RenameTask,
    RenameSubtask,
    AddNotebook,
    EditNotebook,
    AddTaskBefore,
    AddTaskAfter,
    EditTask,
    InspectTask,
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
    // Inspector (Add/Edit Mode)
    pub inspector: Inspector,
}

use ratatui::layout::{Constraint, Direction, Layout};

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

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match &mut self.mode {
            AppMode::Overview => self.overview.render(f, area, true),
            AppMode::NotebookDetail => self.nb_detail.render(f, area),

            AppMode::Add(action) => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .spacing(1)
                    .split(area);

                match action {
                    PendingAction::AddNotebook | PendingAction::EditNotebook => {
                        self.overview.render(f, chunks[0], false);
                    }
                    PendingAction::AddTaskBefore
                    | PendingAction::AddTaskAfter
                    | PendingAction::EditTask
                    | PendingAction::InspectTask => {
                        self.nb_detail.render(f, chunks[0]);
                    }
                    _ => {}
                }

                self.inspector.render(f, chunks[1]);
            }

            m if m.is_popup() => {
                // Render the window in the background
                match &self.last_window {
                    AppMode::Overview => self.overview.render(f, area, true),
                    AppMode::NotebookDetail => self.nb_detail.render(f, area),

                    _ => {}
                }
                // Render the popup ontop
                match &mut self.mode {
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

        // Mode-specific keys
        let current_mode = self.mode.clone();
        match current_mode {
            AppMode::Overview => self.overview_handle_input(key),
            AppMode::NotebookDetail => self.notebook_detail_handle_input(key),
            AppMode::Add(action) => self.inspector_handle_input(action, key),
            AppMode::Rename(popup, action) => self.handle_rename(popup, action, key),
            AppMode::Confirm(popup, action) => {
                let p = popup;
                if let Some(confirmed) = p.handle_input(key) {
                    if confirmed {
                        match action {
                            PendingAction::DeleteNotebook => self.delete_selected_notebook(),
                            PendingAction::DeleteTask => self.delete_selected_task(),
                            PendingAction::DeleteSubtask => self.delete_selected_subtask(),
                            _ => {}
                        }
                        self.mode = self.last_window.clone();
                    } else {
                        // If they said NO to discarding changes, put them back in Add mode
                        match action {
                            PendingAction::AddNotebook
                            | PendingAction::EditNotebook
                            | PendingAction::AddTaskBefore
                            | PendingAction::AddTaskAfter
                            | PendingAction::EditTask => {
                                self.mode = AppMode::Add(action);
                            }
                            _ => {
                                self.mode = self.last_window.clone();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn add_notebook(&mut self) {
        self.last_window = self.mode.clone();
        self.inspector = Inspector::setup(None, None, String::from("Tasks"));
        self.mode = AppMode::Add(PendingAction::AddNotebook);
    }

    pub fn edit_notebook(&mut self) {
        if let Some(idx) = self.overview.state.selected() {
            self.last_window = self.mode.clone();
            let notebook = &self.notebooks[idx];
            self.inspector = Inspector::setup(Some(notebook), None, String::from("Tasks"));
            self.mode = AppMode::Add(PendingAction::EditNotebook);
        }
    }

    pub fn inspect_task(&mut self) {
        use crate::ui::inspect_window::InspectMode;
        if let Some(nb) = &self.nb_detail.notebook {
            if let Some(idx) = self.nb_detail.selected_task_idx {
                self.last_window = self.mode.clone();
                let task = &nb.tasks[idx];
                self.inspector = Inspector::setup(None, Some(task), String::from("Subtasks"));
                self.inspector.mode = InspectMode::View;
                self.mode = AppMode::Add(PendingAction::InspectTask);
            }
        }
    }

    pub fn add_task(&mut self, action: PendingAction) {
        self.last_window = self.mode.clone();
        self.inspector = Inspector::setup(None, None, String::from("Subtasks"));
        self.mode = AppMode::Add(action);
    }

    pub fn edit_task(&mut self) {
        if let Some(nb) = &self.nb_detail.notebook {
            if let Some(idx) = self.nb_detail.selected_task_idx {
                self.last_window = self.mode.clone();
                let task = &nb.tasks[idx];
                self.inspector = Inspector::setup(None, Some(task), String::from("Subtasks"));
                self.mode = AppMode::Add(PendingAction::EditTask);
            }
        }
    }

    pub fn sync_overview(&mut self) {
        self.overview.notebooks = self.notebooks.clone();
        self.overview.sync_inspector();
    }

    pub fn inspector_handle_input(&mut self, action: PendingAction, key: KeyEvent) {
        if let Some(signal) = self.inspector.handle_input(key) {
            match signal {
                InspectorAction::Submit => {
                    match action {
                        PendingAction::AddNotebook => {
                            self.create_new_notebook(
                                self.inspector.title_input.clone(),
                                self.inspector.desc_input.clone(),
                                self.inspector.list_items.clone(),
                            );
                        }
                        PendingAction::EditNotebook => {
                            self.update_existing_notebook(
                                self.inspector.title_input.clone(),
                                self.inspector.desc_input.clone(),
                                self.inspector.list_items.clone(),
                            );
                        }
                        PendingAction::AddTaskBefore => {
                            self.create_new_task(
                                self.inspector.title_input.clone(),
                                self.inspector.desc_input.clone(),
                                self.inspector.list_items.clone(),
                                true, // Before
                            );
                        }
                        PendingAction::AddTaskAfter => {
                            self.create_new_task(
                                self.inspector.title_input.clone(),
                                self.inspector.desc_input.clone(),
                                self.inspector.list_items.clone(),
                                false, // After
                            );
                        }
                        PendingAction::EditTask => {
                            self.update_existing_task(
                                self.inspector.title_input.clone(),
                                self.inspector.desc_input.clone(),
                                self.inspector.list_items.clone(),
                            );
                        }
                        _ => {}
                    }
                    self.mode = self.last_window.clone();
                }
                InspectorAction::Cancel => {
                    if self.inspector.is_empty() || matches!(action, PendingAction::InspectTask) {
                        self.mode = self.last_window.clone();
                    } else {
                        let popup = ConfirmPopup::new(
                            String::from("Discard Changes"),
                            String::from("You have unsaved changes. Discard?"),
                        );
                        self.mode = AppMode::Confirm(popup, action);
                    }
                }
                InspectorAction::Edit => {
                    use crate::ui::inspect_window::InspectMode;
                    self.inspector.mode = InspectMode::Edit;

                    // -- Transition --
                    // If we were inspecting, we are now editing!
                    let new_action = match action {
                        PendingAction::InspectTask => PendingAction::EditTask,
                        _ => action,
                    };
                    self.mode = AppMode::Add(new_action);
                }
            }
        }
    }

    pub fn create_new_task(
        &mut self,
        name: String,
        description: String,
        subtask_names: Vec<String>,
        before: bool,
    ) {
        use crate::models::subtask::Subtask;
        use crate::models::task::Task;
        use crate::ui::task_column::TaskColumnState;

        if let Some(nb) = &mut self.nb_detail.notebook {
            let current_idx = self.nb_detail.selected_task_idx.unwrap_or(0);
            let insert_idx = if before { current_idx } else { current_idx + 1 };

            let mut task = Task {
                name,
                description,
                completion: 0.0,
                is_done: false,
                subtasks: subtask_names
                    .into_iter()
                    .map(|s| Subtask {
                        name: s,
                        is_done: false,
                    })
                    .collect(),
            };
            task.recalculate_completion();

            // Insert into data and UI state
            nb.tasks.insert(insert_idx, task);
            self.nb_detail
                .task_states
                .insert(insert_idx, TaskColumnState::new());
            self.nb_detail.selected_task_idx = Some(insert_idx);

            let _ = self.storage.save_notebook(nb);
            self.notebooks[self.selected_notebook_idx] = nb.clone();
        }
    }

    pub fn update_existing_task(
        &mut self,
        name: String,
        description: String,
        subtask_names: Vec<String>,
    ) {
        use crate::models::subtask::Subtask;
        use std::collections::HashMap;

        if let Some(nb) = &mut self.nb_detail.notebook {
            if let Some(idx) = self.nb_detail.selected_task_idx {
                let task = &mut nb.tasks[idx];
                task.name = name;
                task.description = description;

                // Map old subtask statuses
                let statuses: HashMap<String, bool> = task
                    .subtasks
                    .iter()
                    .map(|s| (s.name.clone(), s.is_done))
                    .collect();

                // Rebuild subtasks while preserving status
                task.subtasks = subtask_names
                    .into_iter()
                    .map(|s_name| {
                        let is_done = *statuses.get(&s_name).unwrap_or(&false);
                        Subtask {
                            name: s_name,
                            is_done,
                        }
                    })
                    .collect();

                task.recalculate_completion();

                let _ = self.storage.save_notebook(nb);
                self.notebooks[self.selected_notebook_idx] = nb.clone();

                // Re-initialize task states to handle new subtask counts
                self.nb_detail.task_states = (0..nb.tasks.len())
                    .map(|_| crate::ui::task_column::TaskColumnState::new())
                    .collect();
            }
        }
    }

    pub fn refresh_notebooks_list(&mut self) {
        // 1. Sync index and load sorted notebooks
        let index = self.storage.validate_and_sync_index().unwrap_or_default();
        let mut notebooks = Vec::new();

        // Save the ID of the currently selected notebook to restore selection later
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

        // 2. Restore selection based on ID
        if let Some(id) = current_id {
            if let Some(new_idx) = self.notebooks.iter().position(|nb| nb.id == id) {
                self.overview.state.select(Some(new_idx));
                self.selected_notebook_idx = new_idx;
            }
        }

        // 3. Sync the active detail view
        if let Some(nb) = self.notebooks.get(self.selected_notebook_idx) {
            self.nb_detail.notebook = Some(nb.clone());
        }

        self.overview.sync_inspector();
    }

    pub fn create_new_notebook(
        &mut self,
        name: String,
        description: String,
        task_names: Vec<String>,
    ) {
        use crate::models::task::Task;
        let mut notebook = Notebook::new(name, description);
        for t_name in task_names {
            notebook.tasks.push(Task {
                name: t_name,
                description: String::new(),
                completion: 0.0,
                is_done: false,
                subtasks: Vec::new(),
            });
        }
        let _ = self.storage.save_notebook(&notebook);
        self.notebooks.push(notebook);
        self.refresh_notebooks_list();
    }

    pub fn update_existing_notebook(
        &mut self,
        name: String,
        description: String,
        task_names: Vec<String>,
    ) {
        if let Some(idx) = self.overview.state.selected() {
            let notebook_id = self.notebooks[idx].id.clone();
            let nb = &mut self.notebooks[idx];
            nb.name = name;
            nb.description = description;

            let _ = self.storage.save_notebook(nb);
            let _ = self.storage.update_last_opened(&notebook_id);
            self.refresh_notebooks_list();
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
                        let notebook_id = self.notebooks[idx].id.clone();

                        // Update time and re-sort immediately
                        let _ = self.storage.update_last_opened(&notebook_id);
                        self.refresh_notebooks_list();

                        // Access the now top-sorted notebook
                        let new_idx = self.selected_notebook_idx;
                        self.nb_detail = NotebookDetail::new(Some(self.notebooks[new_idx].clone()));
                        self.mode = AppMode::NotebookDetail;
                        self.last_window = AppMode::NotebookDetail;
                    }
                }
                OverviewAction::RenameNotebook => {
                    if let Some(idx) = self.overview.state.selected() {
                        let current_name = self.notebooks[idx].name.clone();
                        let popup =
                            RenamePopup::new(String::from("Rename notebook"), current_name, None);
                        self.mode = AppMode::Rename(popup, PendingAction::RenameNotebook);
                    }
                }
                OverviewAction::AddNotebook => self.add_notebook(),
                OverviewAction::EditNotebook => self.edit_notebook(),
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
                        let _ = self.storage.update_last_opened(&nb.id);
                    }
                    self.refresh_notebooks_list();
                    self.mode = AppMode::Overview;
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
                NotebookViewAction::AddTaskBefore => self.add_task(PendingAction::AddTaskBefore),
                NotebookViewAction::AddTaskAfter => self.add_task(PendingAction::AddTaskAfter),
                NotebookViewAction::EditTask => self.edit_task(),
                NotebookViewAction::InspectTask => self.inspect_task(),
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
            AppMode::Overview => true,
            AppMode::NotebookDetail => true,

            _ => false,
        }
    }

    pub fn is_popup(&self) -> bool {
        match self {
            AppMode::Rename(_, _) => true,
            AppMode::Confirm(_, _) => true,
            AppMode::Add(_) => true,
            AppMode::Help => true,

            _ => false,
        }
    }
}
