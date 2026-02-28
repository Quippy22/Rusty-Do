use crossterm::event::{KeyCode, KeyEvent};

use crate::app::actions::{self, PendingAction};
use crate::app::{App, AppMode};

use crate::ui::{
    confirm::ConfirmPopup, inspect_window::InspectorAction, notebook_detail::NotebookViewAction,
    overview::OverviewAction, rename::RenamePopup,
};

pub fn handle_input(app: &mut App, key: KeyEvent) {
    // -- Global --
    if key.code == KeyCode::Char('q') && app.mode.can_quit() {
        app.quit();
        return;
    }

    // -- Mode Routing --
    match app.mode.clone() {
        AppMode::Overview => handle_overview(app, key),
        AppMode::NotebookDetail => handle_detail(app, key),
        AppMode::Add(action) => handle_inspector(app, action, key),
        AppMode::Confirm(popup, action) => handle_confirm(app, popup, action, key),
        AppMode::Rename(popup, action) => handle_rename(app, popup, action, key),
    }
}

fn handle_overview(app: &mut App, key: KeyEvent) {
    if let Some(idx) = app.overview.state.selected() {
        app.selected_notebook_idx = idx;
    }

    if let Some(action) = app.overview.handle_input(key) {
        match action {
            OverviewAction::AddNotebook => actions::init_add_notebook(app),
            OverviewAction::EditNotebook => actions::init_edit_notebook(app),
            OverviewAction::AccessNotebook => actions::enter_notebook(app),
            OverviewAction::DeleteNotebook => {
                let name = app.notebooks[app.selected_notebook_idx].name.clone();
                let popup =
                    ConfirmPopup::new(String::from("Delete Notebook"), format!("Delete {}?", name));
                app.mode = AppMode::Confirm(popup, PendingAction::DeleteNotebook);
            }
            OverviewAction::RenameNotebook => {
                let name = app.notebooks[app.selected_notebook_idx].name.clone();
                let popup = RenamePopup::new(String::from("Rename Notebook"), name, PendingAction::RenameNotebook);
                app.mode = AppMode::Rename(popup, PendingAction::RenameNotebook);
            }
        }
    }
}

fn handle_detail(app: &mut App, key: KeyEvent) {
    if let Some(action) = app.nb_detail.handle_input(key) {
        match action {
            NotebookViewAction::Exit => actions::exit_notebook(app),
            NotebookViewAction::AddTaskBefore => {
                actions::init_add_task(app, PendingAction::AddTaskBefore)
            }
            NotebookViewAction::AddTaskAfter => {
                actions::init_add_task(app, PendingAction::AddTaskAfter)
            }
            NotebookViewAction::EditTask => actions::init_edit_task(app),
            NotebookViewAction::InspectTask => actions::init_inspect_task(app),

            // -- Renames --
            NotebookViewAction::RenameTask => {
                if let Some(nb) = &app.nb_detail.notebook {
                    if let Some(idx) = app.nb_detail.selected_task_idx {
                        let current_name = nb.tasks[idx].name.clone();
                        let popup =
                            RenamePopup::new(String::from("Rename Task"), current_name, PendingAction::RenameTask);
                        app.mode = AppMode::Rename(popup, PendingAction::RenameTask);
                    }
                }
            }
            NotebookViewAction::RenameSubtask => {
                if let Some(nb) = &app.nb_detail.notebook {
                    if let Some(t_idx) = app.nb_detail.selected_task_idx {
                        if let Some(s_idx) = app.nb_detail.task_states[t_idx].state.selected() {
                            let current_name = nb.tasks[t_idx].subtasks[s_idx].name.clone();
                            let popup = RenamePopup::new(
                                String::from("Rename Subtask"),
                                current_name,
                                PendingAction::RenameSubtask,
                            );
                            app.mode = AppMode::Rename(popup, PendingAction::RenameSubtask);
                        }
                    }
                }
            }

            // -- Deletes --
            NotebookViewAction::DeleteTask => {
                if let Some(nb) = &app.nb_detail.notebook {
                    if let Some(idx) = app.nb_detail.selected_task_idx {
                        let name = nb.tasks[idx].name.clone();
                        let popup = ConfirmPopup::new(
                            String::from("Delete Task"),
                            format!("Delete {}?", name),
                        );
                        app.mode = AppMode::Confirm(popup, PendingAction::DeleteTask);
                    }
                }
            }
            NotebookViewAction::DeleteSubtask => {
                if let Some(nb) = &app.nb_detail.notebook {
                    if let Some(t_idx) = app.nb_detail.selected_task_idx {
                        if let Some(s_idx) = app.nb_detail.task_states[t_idx].state.selected() {
                            let name = nb.tasks[t_idx].subtasks[s_idx].name.clone();
                            let popup = ConfirmPopup::new(
                                String::from("Delete Subtask"),
                                format!("Delete {}?", name),
                            );
                            app.mode = AppMode::Confirm(popup, PendingAction::DeleteSubtask);
                        }
                    }
                }
            }
            NotebookViewAction::ConfirmToggleTask => {
                if let Some(nb) = &app.nb_detail.notebook {
                    if let Some(t_idx) = app.nb_detail.selected_task_idx {
                        let name = nb.tasks[t_idx].name.clone();
                        let popup = ConfirmPopup::new(
                            String::from("Toggle Task"),
                            format!("Toggle completion for {}?", name),
                        );
                        app.mode = AppMode::Confirm(popup, PendingAction::ToggleTask);
                    }
                }
            }
        }
    }
}

fn handle_inspector(app: &mut App, action: PendingAction, key: KeyEvent) {
    if let Some(signal) = app.inspector.handle_input(key) {
        match signal {
            InspectorAction::Submit => actions::submit_inspector(app, action),
            InspectorAction::Cancel => {
                if matches!(action, PendingAction::InspectTask) || app.inspector.is_empty() {
                    app.mode = app.last_window.clone();
                } else {
                    let popup = ConfirmPopup::new(
                        String::from("Discard Changes"),
                        String::from("Discard unsaved text?"),
                    );
                    app.mode = AppMode::Confirm(popup, action);
                }
            }
            InspectorAction::Edit => {
                // Transition logic moved to actions or kept here? Keep here for now.
                app.inspector.mode = crate::ui::inspect_window::InspectMode::Edit;
                let new_action = match action {
                    PendingAction::InspectTask => PendingAction::EditTask,
                    _ => action,
                };
                app.mode = AppMode::Add(new_action);
            }
        }
    }
}

fn handle_confirm(app: &mut App, popup: ConfirmPopup, action: PendingAction, key: KeyEvent) {
    if let Some(confirmed) = popup.handle_input(key) {
        if confirmed {
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
                        let s_idx =
                            t_idx.and_then(|i| app.nb_detail.task_states[i].state.selected());

                        if let Some(updated) =
                            actions::delete_element(action.clone(), nb, t_idx, s_idx)
                        {
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
                _ => {}
            }
            app.mode = app.last_window.clone();
        } else {
            // Stay in Add/Edit mode if it was a discard confirmation
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
    }
}

fn handle_rename(app: &mut App, mut popup: RenamePopup, action: PendingAction, key: KeyEvent) {
    if let Some(save) = popup.handle_input(key) {
        if save {
            let new_name = popup.input.clone();
            match action {
                PendingAction::RenameNotebook => {
                    if app.notebooks.iter().any(|n| n.name == new_name) {
                        popup.warning = String::from("Name already used.");
                        app.mode = AppMode::Rename(popup, action);
                        return;
                    }
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
            app.mode = app.last_window.clone();
        } else {
            app.mode = app.last_window.clone();
        }
    } else {
        app.mode = AppMode::Rename(popup, action);
    }
}
