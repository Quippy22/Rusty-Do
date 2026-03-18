use std::collections::HashMap;

use crate::app::{App, AppMode};
use crate::models::{notebook::Notebook, subtask::Subtask, task::Task};
use crate::ui::confirm::ConfirmPopup;
use crate::ui::inspect_window::{InspectMode, Inspector};
use crate::ui::notebook_detail::NotebookDetail;
use crate::ui::rename::RenamePopup;

#[derive(Clone, PartialEq, Debug)]
pub enum PendingAction {
    // Deletes
    DeleteNotebook,
    DeleteTask,
    DeleteSubtask,
    // Renames
    RenameNotebook,
    RenameTask,
    RenameSubtask,
    // Adds
    AddNotebook,
    AddTaskBefore,
    AddTaskAfter,
    AddSubtaskBefore,
    AddSubtaskAfter,
    // Edits
    EditTask,
    EditNotebook,

    InspectTask,
    ToggleTask,
}

// -- Deletions --
pub fn delete_element(
    action: PendingAction,
    mut notebook: Notebook,
    task_idx: Option<usize>,
    subtask_idx: Option<usize>,
) -> Option<Notebook> {
    // A universal delete function for internal notebook elements.
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

// -- Prompts --
pub fn prompt_delete(app: &mut App, action: PendingAction) {
    let (title, name) = match action {
        PendingAction::DeleteNotebook => {
            let name = app
                .notebooks
                .get(app.selected_notebook_idx)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| "Notebook".to_string());
            ("Delete Notebook", name)
        }
        PendingAction::DeleteTask => {
            let name = app
                .nb_detail
                .notebook
                .as_ref()
                .and_then(|nb| {
                    app.nb_detail
                        .selected_task_idx
                        .and_then(|idx| nb.tasks.get(idx))
                })
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Task".to_string());
            ("Delete Task", name)
        }
        PendingAction::DeleteSubtask => {
            let name = app
                .nb_detail
                .notebook
                .as_ref()
                .and_then(|nb| {
                    app.nb_detail.selected_task_idx.and_then(|t_idx| {
                        app.nb_detail.task_states.get(t_idx).and_then(|state| {
                            state.state.selected().and_then(|s_idx| {
                                nb.tasks.get(t_idx).and_then(|t| t.subtasks.get(s_idx))
                            })
                        })
                    })
                })
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "Subtask".to_string());
            ("Delete Subtask", name)
        }
        _ => ("Delete", String::new()),
    };
    let popup = ConfirmPopup::new(
        title.into(),
        format!("Delete {}?", name),
        vec!["Yes".to_string(), "No".to_string()],
    );
    app.mode = AppMode::Confirm(popup, action);
}

pub fn prompt_rename(app: &mut App, action: PendingAction) {
    let (title, name) = match action {
        PendingAction::RenameNotebook => {
            let name = app
                .notebooks
                .get(app.selected_notebook_idx)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| "Notebook".to_string());
            ("Rename Notebook", name)
        }
        PendingAction::RenameTask => {
            let name = app
                .nb_detail
                .notebook
                .as_ref()
                .and_then(|nb| {
                    app.nb_detail
                        .selected_task_idx
                        .and_then(|idx| nb.tasks.get(idx))
                })
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Task".to_string());
            ("Rename Task", name)
        }
        PendingAction::RenameSubtask => {
            let name = app
                .nb_detail
                .notebook
                .as_ref()
                .and_then(|nb| {
                    app.nb_detail.selected_task_idx.and_then(|t_idx| {
                        app.nb_detail.task_states.get(t_idx).and_then(|state| {
                            state.state.selected().and_then(|s_idx| {
                                nb.tasks.get(t_idx).and_then(|t| t.subtasks.get(s_idx))
                            })
                        })
                    })
                })
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "Subtask".to_string());
            ("Rename Subtask", name)
        }
        _ => ("Rename", String::new()),
    };
    let popup = RenamePopup::new(title.into(), name, action.clone());
    app.mode = AppMode::Rename(popup, action);
}

pub fn prompt_toggle_task(app: &mut App) {
    if let Some(nb) = &app.nb_detail.notebook {
        if let Some(idx) = app.nb_detail.selected_task_idx {
            let name = nb.tasks[idx].name.clone();
            let popup = ConfirmPopup::new(
                "Toggle Task".into(),
                format!("Toggle completion for {}?", name),
                vec!["Yes".to_string(), "No".to_string()],
            );
            app.mode = AppMode::Confirm(popup, PendingAction::ToggleTask);
        }
    }
}

pub fn show_help(app: &mut App) {
    if !app.mode.is_popup() {
        app.last_window = app.mode.clone();
    }
    app.mode = AppMode::Help;
}

pub fn exit_help(app: &mut App) {
    app.mode = app.last_window.clone();

    // Restore last_window to the base view if we are returning to the Inspector,
    // breaking the infinite 'Esc' loop.
    if let AppMode::Add(action) = &app.mode {
        match action {
            PendingAction::AddNotebook | PendingAction::EditNotebook => {
                app.last_window = AppMode::Overview;
            }
            _ => {
                app.last_window = AppMode::NotebookDetail;
            }
        }
    }
}

pub fn cycle_theme(app: &mut App) {
    app.theme_idx = (app.theme_idx + 1) % 2;
    let new_theme = match app.theme_idx {
        0 => crate::ui::theme::Theme::default(),
        1 => crate::ui::theme::Theme::nord(),
        _ => crate::ui::theme::Theme::default(),
    };
    crate::ui::theme::set_theme(new_theme);

    // Persist the choice
    app.storage.save_settings(&crate::storage::persistence::AppSettings {
        theme_idx: app.theme_idx,
    });
}

// -- Notebook Actions --
pub fn add_notebook(app: &mut App) {
    app.last_window = app.mode.clone();

    // Create Ghost placeholder
    let placeholder = create_notebook_struct("New Notebook", "", &[]);
    app.notebooks.push(placeholder);
    app.selected_notebook_idx = app.notebooks.len() - 1;
    app.overview.notebooks = app.notebooks.clone();
    app.overview.state.select(Some(app.selected_notebook_idx));

    app.inspector = Inspector::setup(None, None, String::from("Tasks"));
    app.mode = AppMode::Add(PendingAction::AddNotebook);
}

pub fn edit_notebook(app: &mut App) {
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

// -- Swapping --

pub fn swap_task(app: &mut App, dir: i32) {
    if let Some(mut nb) = app.nb_detail.notebook.clone() {
        if let Some(t_idx) = app.nb_detail.selected_task_idx {
            let target_idx = (t_idx as i32 + dir).max(0).min(nb.tasks.len() as i32 - 1) as usize;
            if t_idx != target_idx {
                nb.tasks.swap(t_idx, target_idx);
                app.nb_detail.selected_task_idx = Some(target_idx);
                app.nb_detail.task_states.swap(t_idx, target_idx);
                app.refresh_nb_detail(nb);
            }
        }
    }
}

pub fn swap_subtask(app: &mut App, dir: i32) {
    if let Some(mut nb) = app.nb_detail.notebook.clone() {
        if let Some(t_idx) = app.nb_detail.selected_task_idx {
            if let Some(s_idx) = app.nb_detail.task_states[t_idx].state.selected() {
                let target_idx = (s_idx as i32 + dir)
                    .max(0)
                    .min(nb.tasks[t_idx].subtasks.len() as i32 - 1)
                    as usize;

                if s_idx != target_idx {
                    nb.tasks[t_idx].subtasks.swap(s_idx, target_idx);

                    // Update selection in state BEFORE refreshing detail
                    app.nb_detail.task_states[t_idx]
                        .state
                        .select(Some(target_idx));

                    app.refresh_nb_detail(nb);
                }
            }
        }
    }
}

// -- Task Actions --
pub fn add_task(app: &mut App, action: PendingAction) {
    if let Some(nb) = &mut app.nb_detail.notebook {
        app.last_window = app.mode.clone();

        // Create Ghost placeholder
        let insert_idx = if nb.tasks.is_empty() {
            0
        } else {
            let current_idx = app.nb_detail.selected_task_idx.unwrap_or(0);
            if action == PendingAction::AddTaskBefore {
                current_idx
            } else {
                current_idx + 1
            }
        };

        let task = create_task_struct("New Task", "", &[]);
        nb.tasks.insert(insert_idx, task);
        app.nb_detail
            .task_states
            .insert(insert_idx, crate::ui::task_column::TaskColumnState::new());
        app.nb_detail.selected_task_idx = Some(insert_idx);

        // Sync master list
        app.notebooks[app.selected_notebook_idx] = nb.clone();

        app.inspector = Inspector::setup(None, None, String::from("Subtasks"));
        app.mode = AppMode::Add(action);
    }
}

pub fn edit_task(app: &mut App) {
    if let Some(nb) = &app.nb_detail.notebook {
        if let Some(idx) = app.nb_detail.selected_task_idx {
            app.last_window = app.mode.clone();
            let task = &nb.tasks[idx];
            app.inspector = Inspector::setup(None, Some(task), String::from("Subtasks"));
            app.mode = AppMode::Add(PendingAction::EditTask);
        }
    }
}

pub fn inspect_task(app: &mut App) {
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

// -- Executioners --
pub fn confirm_success(app: &mut App, action: PendingAction) {
    // Perform cleanup first (e.g. discarding ghosts)
    cleanup_ghost(app, action.clone());

    match action {
        PendingAction::DeleteNotebook => {
            let id = app.notebooks[app.selected_notebook_idx].id.clone();
            let _ = std::fs::remove_file(app.storage.fs.get_notebook_path(&id));
            app.notebooks.remove(app.selected_notebook_idx);
            let _ = app.storage.validate_and_sync_index();
            app.refresh_notebooks_list();
        }
        PendingAction::DeleteTask | PendingAction::DeleteSubtask => {
            if let Some(nb) = app.nb_detail.notebook.clone() {
                let t_idx = app.nb_detail.selected_task_idx;
                let s_idx = t_idx.and_then(|i| app.nb_detail.task_states.get(i).and_then(|ts| ts.state.selected()));

                if let Some(updated) = delete_element(action.clone(), nb, t_idx, s_idx) {
                    // Sync the nb_detail UI state BEFORE the refresh
                    if action == PendingAction::DeleteTask {
                        if let Some(idx) = t_idx {
                            if idx < app.nb_detail.task_states.len() {
                                app.nb_detail.task_states.remove(idx);
                            }
                            let new_len = updated.tasks.len();
                            if new_len == 0 {
                                app.nb_detail.selected_task_idx = None;
                                app.nb_detail.scroll_offset = 0;
                            } else {
                                app.nb_detail.selected_task_idx = Some(idx.min(new_len - 1));
                                app.nb_detail.scroll_offset = app.nb_detail.scroll_offset.min(new_len - 1);
                            }
                        }
                    } else if action == PendingAction::DeleteSubtask {
                        if let (Some(ti), Some(si)) = (t_idx, s_idx) {
                            if ti < updated.tasks.len() {
                                let new_sub_len = updated.tasks[ti].subtasks.len();
                                if new_sub_len == 0 {
                                    app.nb_detail.task_states[ti].state.select(None);
                                } else {
                                    app.nb_detail.task_states[ti].state.select(Some(si.min(new_sub_len - 1)));
                                }
                            }
                        }
                    }
                    app.refresh_nb_detail(updated);
                }
            }
        }
        PendingAction::ToggleTask => {
            if let Some(mut nb) = app.nb_detail.notebook.clone() {
                if let Some(idx) = app.nb_detail.selected_task_idx {
                    nb.tasks[idx].toggle_task();
                    app.refresh_nb_detail(nb);
                }
            }
        }
        PendingAction::EditTask | PendingAction::InspectTask | PendingAction::EditNotebook => {
            // Discarding edits means we must revert the real-time memory mutations
            // by reloading the unmutated state from disk.
            app.refresh_notebooks_list();
        }
        _ => {}
    }
    app.mode = app.last_window.clone();
}

pub fn prompt_discard_changes(app: &mut App, action: PendingAction) {
    let is_dirty = match action {
        PendingAction::AddNotebook | PendingAction::AddTaskBefore | PendingAction::AddTaskAfter => {
            let default_name = if matches!(action, PendingAction::AddNotebook) {
                "New Notebook"
            } else {
                "New Task"
            };
            app.inspector.title_input != default_name
                || !app.inspector.desc_input.is_empty()
                || !app.inspector.list_items.is_empty()
                || !app.inspector.task_input.trim().is_empty()
        }
        PendingAction::EditNotebook => {
            let id = &app.notebooks[app.selected_notebook_idx].id;
            if let Ok(disk_nb) = app.storage.load_notebook(id) {
                let task_names: Vec<String> =
                    disk_nb.tasks.iter().map(|t| t.name.clone()).collect();
                app.inspector.title_input != disk_nb.name
                    || app.inspector.desc_input != disk_nb.description
                    || app.inspector.list_items != task_names
            } else {
                true // Fallback to dirty if disk read fails
            }
        }
        PendingAction::EditTask | PendingAction::InspectTask => {
            if let Some(nb) = &app.nb_detail.notebook {
                if let Ok(disk_nb) = app.storage.load_notebook(&nb.id) {
                    if let Some(idx) = app.nb_detail.selected_task_idx {
                        if idx < disk_nb.tasks.len() {
                            let task = &disk_nb.tasks[idx];
                            let sub_names: Vec<String> =
                                task.subtasks.iter().map(|s| s.name.clone()).collect();
                            app.inspector.title_input != task.name
                                || app.inspector.desc_input != task.description
                                || app.inspector.list_items != sub_names
                                || !app.inspector.task_input.trim().is_empty()
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    true
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if is_dirty {
        let popup = ConfirmPopup::new(
            String::from("Unsaved Changes"),
            String::from("You have unsaved text. What would you like to do?"),
            vec![
                "Save & Exit".to_string(),
                "Discard".to_string(),
                "Cancel".to_string(),
            ],
        );
        app.mode = AppMode::Confirm(popup, action);
    } else {
        cleanup_ghost(app, action);
        app.mode = app.last_window.clone();
    }
}

pub fn transition_to_edit(app: &mut App, action: PendingAction) {
    app.inspector.mode = InspectMode::Edit;
    let new_action = match action {
        PendingAction::InspectTask => PendingAction::EditTask,
        _ => action,
    };
    app.mode = AppMode::Add(new_action);
}

pub fn confirm_cancel(app: &mut App, action: PendingAction) {
    match action {
        PendingAction::AddNotebook
        | PendingAction::EditNotebook
        | PendingAction::AddTaskBefore
        | PendingAction::AddTaskAfter
        | PendingAction::EditTask => {
            app.mode = AppMode::Add(action);
        }
        _ => app.mode = app.last_window.clone(),
    }
}

pub fn sync_inspector_title(app: &mut App, action: PendingAction) {
    let title = app.inspector.title_input.clone();
    match action {
        PendingAction::AddNotebook | PendingAction::EditNotebook => {
            app.notebooks[app.selected_notebook_idx].name = title;
            app.overview.notebooks = app.notebooks.clone();
        }
        PendingAction::AddTaskBefore | PendingAction::AddTaskAfter | PendingAction::EditTask => {
            if let Some(nb) = &mut app.nb_detail.notebook {
                if let Some(idx) = app.nb_detail.selected_task_idx {
                    nb.tasks[idx].name = title;
                    app.notebooks[app.selected_notebook_idx] = nb.clone();
                }
            }
        }
        _ => {}
    }
}

pub fn submit_inspector(app: &mut App, action: PendingAction) {
    let description = app.inspector.desc_input.clone();

    match action {
        PendingAction::AddNotebook => {
            // Finalize the Ghost
            let id = {
                let nb = &mut app.notebooks[app.selected_notebook_idx];
                nb.description = description;
                nb.tasks = app
                    .inspector
                    .list_items
                    .iter()
                    .map(|t| create_task_struct(t, "", &[]))
                    .collect();

                let _ = app.storage.save_notebook(nb);
                nb.id.clone()
            };
            let _ = app.storage.update_last_opened(&id);
        }
        PendingAction::EditNotebook => {
            if let Some(idx) = app.overview.state.selected() {
                let nb = &mut app.notebooks[idx];
                nb.name = app.inspector.title_input.clone();
                nb.description = description;
                let _ = app.storage.save_notebook(nb);
                let _ = app.storage.update_last_opened(&nb.id);
            }
        }
        PendingAction::EditTask | PendingAction::AddTaskBefore | PendingAction::AddTaskAfter => {
            if let Some(nb) = &mut app.nb_detail.notebook {
                if let Some(idx) = app.nb_detail.selected_task_idx {
                    // Update the existing placeholder/task
                    let task = nb.tasks.remove(idx);
                    let updated = update_task_struct(
                        task,
                        &app.inspector.title_input,
                        &description,
                        &app.inspector.list_items,
                    );
                    nb.tasks.insert(idx, updated);

                    let _ = app.storage.save_notebook(nb);
                    app.notebooks[app.selected_notebook_idx] = nb.clone();
                }
            }
        }
        _ => {}
    }
    app.refresh_notebooks_list();
    app.mode = app.last_window.clone();
}

pub fn add_subtask(app: &mut App, name: String, action: PendingAction) {
    if let Some(mut nb) = app.nb_detail.notebook.clone() {
        if let Some(t_idx) = app.nb_detail.selected_task_idx {
            let current_s_idx = app.nb_detail.task_states[t_idx]
                .state
                .selected()
                .unwrap_or(0);
            let insert_idx = if action == PendingAction::AddSubtaskBefore {
                current_s_idx
            } else if nb.tasks[t_idx].subtasks.is_empty() {
                0
            } else {
                current_s_idx + 1
            };

            nb.tasks[t_idx].subtasks.insert(
                insert_idx,
                Subtask {
                    name,
                    is_done: false,
                },
            );
            nb.tasks[t_idx].recalculate_completion();
            app.refresh_nb_detail(nb);

            // Restore selection to the new subtask
            app.nb_detail.task_states[t_idx]
                .state
                .select(Some(insert_idx));
        }
    }
}

pub fn apply_rename(app: &mut App, new_name: String, action: PendingAction) {
    match action {
        PendingAction::RenameNotebook => {
            app.notebooks[app.selected_notebook_idx].name = new_name;
            let nb = &app.notebooks[app.selected_notebook_idx];
            let _ = app.storage.save_notebook(nb);
            app.refresh_notebooks_list();
        }
        PendingAction::RenameTask => {
            if let Some(mut nb) = app.nb_detail.notebook.clone() {
                if let Some(idx) = app.nb_detail.selected_task_idx {
                    nb.tasks[idx].name = new_name;
                    app.refresh_nb_detail(nb);
                }
            }
        }
        PendingAction::RenameSubtask => {
            if let Some(mut nb) = app.nb_detail.notebook.clone() {
                if let Some(t_idx) = app.nb_detail.selected_task_idx {
                    if let Some(s_idx) = app.nb_detail.task_states[t_idx].state.selected() {
                        nb.tasks[t_idx].subtasks[s_idx].name = new_name;
                        app.refresh_nb_detail(nb);
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn cleanup_ghost(app: &mut App, action: PendingAction) {
    match action {
        PendingAction::AddNotebook => {
            if app.notebooks.len() > app.selected_notebook_idx {
                app.notebooks.remove(app.selected_notebook_idx);
                app.refresh_notebooks_list();
            }
        }
        PendingAction::AddTaskBefore | PendingAction::AddTaskAfter => {
            if let Some(nb) = &mut app.nb_detail.notebook {
                if let Some(idx) = app.nb_detail.selected_task_idx {
                    if idx < nb.tasks.len() {
                        nb.tasks.remove(idx);
                        app.nb_detail.task_states.remove(idx);

                        let new_len = nb.tasks.len();
                        if new_len == 0 {
                            app.nb_detail.selected_task_idx = None;
                            app.nb_detail.scroll_offset = 0;
                        } else {
                            app.nb_detail.selected_task_idx = Some(idx.min(new_len - 1));
                            app.nb_detail.scroll_offset =
                                app.nb_detail.scroll_offset.min(new_len - 1);
                        }

                        app.notebooks[app.selected_notebook_idx] = nb.clone();
                    }
                }
            }
        }
        _ => {}
    }
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
