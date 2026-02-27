use std::collections::HashMap;

use crate::app::{App, AppMode};
use crate::models::{notebook::Notebook, subtask::Subtask, task::Task};
use crate::ui::inspect_window::{InspectMode, Inspector};
use crate::ui::notebook_detail::NotebookDetail;

#[derive(Clone, PartialEq, Debug)]
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

/// --- Deletions ---

/// A universal delete function for internal notebook elements.
/// Returns Some(Notebook) if modified, None if the notebook itself should be deleted.
pub fn delete_element(
    action: PendingAction,
    mut notebook: Notebook,
    task_idx: Option<usize>,
    subtask_idx: Option<usize>,
) -> Option<Notebook> {
    match action {
        PendingAction::DeleteNotebook => None,

        PendingAction::DeleteTask => {
            if let Some(t_idx) = task_idx {
                if t_idx < notebook.tasks.len() {
                    notebook.tasks.remove(t_idx);
                }
            }
            Some(notebook)
        }

        PendingAction::DeleteSubtask => {
            if let (Some(t_idx), Some(s_idx)) = (task_idx, subtask_idx) {
                if t_idx < notebook.tasks.len() && s_idx < notebook.tasks[t_idx].subtasks.len() {
                    notebook.tasks[t_idx].subtasks.remove(s_idx);
                    notebook.tasks[t_idx].recalculate_completion();
                }
            }
            Some(notebook)
        }

        _ => Some(notebook),
    }
}

// -- Notebook Actions --

pub fn init_add_notebook(app: &mut App) {
    app.last_window = app.mode.clone();
    app.inspector = Inspector::setup(None, None, String::from("Tasks"));
    app.mode = AppMode::Add(PendingAction::AddNotebook);
}

pub fn init_edit_notebook(app: &mut App) {
    if let Some(idx) = app.overview.state.selected() {
        app.last_window = app.mode.clone();
        let notebook = &app.notebooks[idx];
        app.inspector = Inspector::setup(Some(notebook), None, String::from("Tasks"));
        app.mode = AppMode::Add(PendingAction::EditNotebook);
    }
}

pub fn enter_notebook(app: &mut App) {
    if let Some(idx) = app.overview.state.selected() {
        let id = app.notebooks[idx].id.clone();
        let _ = app.storage.update_last_opened(&id);
        app.refresh_notebooks_list();

        // Refresh selects 0, so we load that
        app.nb_detail = NotebookDetail::new(Some(app.notebooks[0].clone()));
        app.mode = AppMode::NotebookDetail;
        app.last_window = AppMode::NotebookDetail;
    }
}

pub fn exit_notebook(app: &mut App) {
    if let Some(nb) = &app.nb_detail.notebook {
        let _ = app.storage.save_notebook(nb);
        let _ = app.storage.update_last_opened(&nb.id);
    }
    app.refresh_notebooks_list();
    app.mode = AppMode::Overview;
    app.last_window = AppMode::Overview;
}

// -- Task Actions --

pub fn init_add_task(app: &mut App, action: PendingAction) {
    app.last_window = app.mode.clone();
    app.inspector = Inspector::setup(None, None, String::from("Subtasks"));
    app.mode = AppMode::Add(action);
}

pub fn init_edit_task(app: &mut App) {
    if let Some(nb) = &app.nb_detail.notebook {
        if let Some(idx) = app.nb_detail.selected_task_idx {
            app.last_window = app.mode.clone();
            let task = &nb.tasks[idx];
            app.inspector = Inspector::setup(None, Some(task), String::from("Subtasks"));
            app.mode = AppMode::Add(PendingAction::EditTask);
        }
    }
}

pub fn init_inspect_task(app: &mut App) {
    if let Some(nb) = &app.nb_detail.notebook {
        if let Some(idx) = app.nb_detail.selected_task_idx {
            app.last_window = app.mode.clone();
            let task = &nb.tasks[idx];
            app.inspector = Inspector::setup(None, Some(task), String::from("Subtasks"));
            app.inspector.mode = InspectMode::View;
            app.mode = AppMode::Add(PendingAction::InspectTask);
        }
    }
}

// -- Inspector Submission --

pub fn submit_inspector(app: &mut App, action: PendingAction) {
    match action {
        PendingAction::AddNotebook => {
            let nb = create_notebook_struct(
                &app.inspector.title_input,
                &app.inspector.desc_input,
                &app.inspector.list_items,
            );
            let _ = app.storage.save_notebook(&nb);
            app.notebooks.push(nb);
        }
        PendingAction::EditNotebook => {
            if let Some(idx) = app.overview.state.selected() {
                let nb = &mut app.notebooks[idx];
                nb.name = app.inspector.title_input.clone();
                nb.description = app.inspector.desc_input.clone();
                let _ = app.storage.save_notebook(nb);
            }
        }
        PendingAction::EditTask => {
            if let Some(nb) = &mut app.nb_detail.notebook {
                if let Some(idx) = app.nb_detail.selected_task_idx {
                    let task = nb.tasks.remove(idx);
                    let updated = update_task_struct(
                        task,
                        &app.inspector.title_input,
                        &app.inspector.desc_input,
                        &app.inspector.list_items,
                    );
                    nb.tasks.insert(idx, updated);
                    let _ = app.storage.save_notebook(nb);
                    app.notebooks[app.selected_notebook_idx] = nb.clone();
                }
            }
        }
        PendingAction::AddTaskBefore | PendingAction::AddTaskAfter => {
            if let Some(nb) = &mut app.nb_detail.notebook {
                let current_idx = app.nb_detail.selected_task_idx.unwrap_or(0);
                let insert_idx = if action == PendingAction::AddTaskBefore {
                    current_idx
                } else {
                    current_idx + 1
                };
                let task = create_task_struct(
                    &app.inspector.title_input,
                    &app.inspector.desc_input,
                    &app.inspector.list_items,
                );
                nb.tasks.insert(insert_idx, task);
                app.nb_detail
                    .task_states
                    .insert(insert_idx, crate::ui::task_column::TaskColumnState::new());
                app.nb_detail.selected_task_idx = Some(insert_idx);
                let _ = app.storage.save_notebook(nb);
                app.notebooks[app.selected_notebook_idx] = nb.clone();
            }
        }
        _ => {}
    }
    app.refresh_notebooks_list();
    app.mode = app.last_window.clone();
}

// -- Internal Helpers --

fn create_notebook_struct(name: &str, desc: &str, task_names: &[String]) -> Notebook {
    let mut notebook = Notebook::new(name.to_string(), desc.to_string());
    for t_name in task_names {
        notebook.tasks.push(Task {
            name: t_name.clone(),
            description: String::new(),
            completion: 0.0,
            is_done: false,
            subtasks: Vec::new(),
        });
    }
    notebook
}

fn create_task_struct(name: &str, desc: &str, subtask_names: &[String]) -> Task {
    let mut task = Task {
        name: name.to_string(),
        description: desc.to_string(),
        completion: 0.0,
        is_done: false,
        subtasks: subtask_names
            .iter()
            .map(|s| Subtask {
                name: s.clone(),
                is_done: false,
            })
            .collect(),
    };
    task.recalculate_completion();
    task
}

fn update_task_struct(mut task: Task, name: &str, desc: &str, subtask_names: &[String]) -> Task {
    let statuses: HashMap<String, bool> = task
        .subtasks
        .iter()
        .map(|s| (s.name.clone(), s.is_done))
        .collect();
    task.name = name.to_string();
    task.description = desc.to_string();
    task.subtasks = subtask_names
        .iter()
        .map(|s_name| {
            let is_done = *statuses.get(s_name).unwrap_or(&false);
            Subtask {
                name: s_name.clone(),
                is_done,
            }
        })
        .collect();
    task.recalculate_completion();
    task
}
