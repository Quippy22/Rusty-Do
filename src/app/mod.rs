pub mod actions;
pub mod input;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::actions::PendingAction;
use crate::models::notebook::Notebook;
use crate::storage::{paths::FileSystem, persistence::Persistence};
use crate::ui::{
    confirm::ConfirmPopup, help::HelpPopup, inspect_window::Inspector,
    notebook_detail::NotebookDetail, overview::Overview, rename::RenamePopup,
    theme::Theme,
};

#[derive(Clone)]
pub enum AppMode {
    // -- Windows --
    Overview,
    NotebookDetail,
    Add(PendingAction),

    // -- Popups --
    Confirm(ConfirmPopup, PendingAction),
    Rename(RenamePopup, PendingAction),
    Help,
}

impl AppMode {
    pub fn can_quit(&self) -> bool {
        matches!(self, AppMode::Overview | AppMode::NotebookDetail)
    }

    pub fn is_popup(&self) -> bool {
        matches!(
            self,
            AppMode::Rename(_, _) | AppMode::Confirm(_, _) | AppMode::Help
        )
    }
}

#[derive(Clone)]
pub struct App {
    pub mode: AppMode,
    pub last_window: AppMode, // Transient state for Esc/Cancel
    pub storage: Persistence,
    pub should_quit: bool,
    pub notebooks: Vec<Notebook>,
    pub selected_notebook_idx: usize,
    pub overview: Overview,
    pub nb_detail: NotebookDetail,
    pub inspector: Inspector,
}

impl App {
    pub fn new(filesystem: FileSystem) -> Self {
        let storage = Persistence::new(filesystem);

        // -- Theme Loading --
        let theme = Theme::load(storage.fs.data_dir.join("theme.json"));
        crate::ui::theme::init_theme(theme);

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
        if let Some(nb) = &self.nb_detail.notebook {
            let _ = self.storage.save_notebook(nb);
        }
        self.should_quit = true;
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        match &mut self.mode {
            AppMode::Overview => self.overview.render(f, area, true),
            AppMode::NotebookDetail => self.nb_detail.render(f, area),

            AppMode::Add(_) => self.render_inspector_view(f, area),

            AppMode::Help => {
                if matches!(self.last_window, AppMode::Add(_)) {
                    self.render_inspector_view(f, area);
                } else {
                    match &self.last_window {
                        AppMode::Overview => self.overview.render(f, area, true),
                        AppMode::NotebookDetail => self.nb_detail.render(f, area),
                        _ => {}
                    }
                }
                HelpPopup::render(f, area, &self.last_window);
            }

            m if m.is_popup() => {
                if matches!(self.last_window, AppMode::Add(_)) {
                    self.render_inspector_view(f, area);
                } else {
                    match &self.last_window {
                        AppMode::Overview => self.overview.render(f, area, true),
                        AppMode::NotebookDetail => self.nb_detail.render(f, area),
                        _ => {}
                    }
                }
                match &mut self.mode {
                    AppMode::Confirm(popup, _) => popup.render(f, area),
                    AppMode::Rename(popup, _) => popup.render(f, area),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn render_inspector_view(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(1)
            .split(area);

        self.inspector.render(f, chunks[1]);

        if self.inspector.list_label == "Tasks" {
            self.overview.render(f, chunks[0], false);
        } else {
            self.nb_detail.render(f, chunks[0]);
        }
    }

    // -- Input Handling --
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
        self.notebooks[self.selected_notebook_idx] = notebook.clone();
        self.nb_detail.notebook = Some(notebook.clone());

        // Preserve states unless the task count changed
        if self.nb_detail.task_states.len() != notebook.tasks.len() {
            self.nb_detail.task_states = (0..notebook.tasks.len())
                .map(|_| crate::ui::task_column::TaskColumnState::new())
                .collect();
        }

        let _ = self.storage.save_notebook(&notebook);
    }
}
