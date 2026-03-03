use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::actions::{self, PendingAction};
use crate::app::{App, AppMode};

use crate::ui::{
    inspect_window::InspectorAction, notebook_detail::NotebookViewAction,
    overview::OverviewAction, rename::RenamePopup,
};

pub fn handle_input(app: &mut App, key: KeyEvent) {
    // -- Global Help Exit --
    if matches!(app.mode, AppMode::Help) {
        actions::exit_help(app);
        return;
    }

    // -- Global Theme Switcher --
    if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::ALT) {
        actions::cycle_theme(app);
        return;
    }

    // -- Global Quit --
    if key.code == KeyCode::Char('q') && app.mode.can_quit() {
        app.quit();
        return;
    }

    // -- Global Help Open --
    if key.code == KeyCode::Char('?') {
        actions::show_help(app);
        return;
    }

    // -- Mode Routing --
    match app.mode.clone() {
        AppMode::Overview => handle_overview(app, key),
        AppMode::NotebookDetail => handle_detail(app, key),
        AppMode::Add(action) => handle_inspector(app, action, key),
        AppMode::Confirm(_, action) => handle_confirm(app, action, key),
        AppMode::Rename(popup, action) => handle_rename(app, popup, action, key),
        AppMode::Help => unreachable!("Help handled at top of function"),
    }
}

fn handle_overview(app: &mut App, key: KeyEvent) {
    if let Some(idx) = app.overview.state.selected() {
        app.selected_notebook_idx = idx;
    }

    if let Some(action) = app.overview.handle_input(key) {
        match action {
            OverviewAction::AddNotebook => actions::add_notebook(app),
            OverviewAction::EditNotebook => actions::edit_notebook(app),
            OverviewAction::AccessNotebook => actions::enter_notebook(app),
            OverviewAction::DeleteNotebook => {
                actions::prompt_delete(app, PendingAction::DeleteNotebook)
            }
            OverviewAction::RenameNotebook => {
                actions::prompt_rename(app, PendingAction::RenameNotebook)
            }
        }
    }
}

fn handle_detail(app: &mut App, key: KeyEvent) {
    let shift = key
        .modifiers
        .contains(crossterm::event::KeyModifiers::SHIFT);

    // -- Swapping (Shift + HJKL) --
    if shift {
        match key.code {
            KeyCode::Char('H') | KeyCode::Left => {
                actions::swap_task(app, -1);
                return;
            }
            KeyCode::Char('L') | KeyCode::Right => {
                actions::swap_task(app, 1);
                return;
            }
            KeyCode::Char('J') | KeyCode::Down => {
                actions::swap_subtask(app, 1);
                return;
            }
            KeyCode::Char('K') | KeyCode::Up => {
                actions::swap_subtask(app, -1);
                return;
            }
            _ => {}
        }
    }

    if let Some(action) = app.nb_detail.handle_input(key) {
        match action {
            NotebookViewAction::Exit => actions::exit_notebook(app),
            NotebookViewAction::AddTaskBefore => {
                actions::add_task(app, PendingAction::AddTaskBefore)
            }
            NotebookViewAction::AddTaskAfter => actions::add_task(app, PendingAction::AddTaskAfter),
            NotebookViewAction::AddSubtaskBefore => {
                let popup = RenamePopup::new(
                    String::from("New Subtask"),
                    String::new(),
                    PendingAction::AddSubtaskBefore,
                );
                app.mode = AppMode::Rename(popup, PendingAction::AddSubtaskBefore);
            }
            NotebookViewAction::AddSubtaskAfter => {
                let popup = RenamePopup::new(
                    String::from("New Subtask"),
                    String::new(),
                    PendingAction::AddSubtaskAfter,
                );
                app.mode = AppMode::Rename(popup, PendingAction::AddSubtaskAfter);
            }
            NotebookViewAction::EditTask => actions::edit_task(app),
            NotebookViewAction::InspectTask => actions::inspect_task(app),
            NotebookViewAction::RenameTask => {
                actions::prompt_rename(app, PendingAction::RenameTask)
            }
            NotebookViewAction::RenameSubtask => {
                actions::prompt_rename(app, PendingAction::RenameSubtask)
            }
            NotebookViewAction::DeleteTask => {
                actions::prompt_delete(app, PendingAction::DeleteTask)
            }
            NotebookViewAction::DeleteSubtask => {
                actions::prompt_delete(app, PendingAction::DeleteSubtask)
            }
            NotebookViewAction::ConfirmToggleTask => actions::prompt_toggle_task(app),
        }
    }
}

fn handle_inspector(app: &mut App, action: PendingAction, key: KeyEvent) {
    if let Some(signal) = app.inspector.handle_input(key) {
        match signal {
            InspectorAction::Submit => actions::submit_inspector(app, action.clone()),
            InspectorAction::Cancel => {
                if matches!(action, PendingAction::InspectTask) || app.inspector.is_empty() {
                    actions::cleanup_ghost(app, action);
                    app.mode = app.last_window.clone();
                } else {
                    actions::prompt_discard_changes(app, action);
                }
            }
            InspectorAction::Edit => actions::transition_to_edit(app, action.clone()),
        }
        return;
    }

    actions::sync_inspector_title(app, action);
}

fn handle_confirm(app: &mut App, action: PendingAction, key: KeyEvent) {
    let mut popup = match &app.mode {
        AppMode::Confirm(p, _) => p.clone(),
        _ => return,
    };

    if let Some(button_idx) = popup.handle_input(key) {
        match action {
            PendingAction::AddNotebook
            | PendingAction::EditNotebook
            | PendingAction::AddTaskBefore
            | PendingAction::AddTaskAfter
            | PendingAction::EditTask
            | PendingAction::InspectTask => match button_idx {
                0 => actions::submit_inspector(app, action),
                1 => actions::confirm_success(app, action),
                _ => actions::confirm_cancel(app, action),
            },
            _ => {
                if button_idx == 0 {
                    actions::confirm_success(app, action);
                } else {
                    actions::confirm_cancel(app, action);
                }
            }
        }
    } else {
        app.mode = AppMode::Confirm(popup, action);
    }
}

fn handle_rename(app: &mut App, mut popup: RenamePopup, action: PendingAction, key: KeyEvent) {
    if let Some(save) = popup.handle_input(key) {
        if save {
            let new_name = popup.input.clone();
            match action {
                PendingAction::AddSubtaskBefore | PendingAction::AddSubtaskAfter => {
                    actions::add_subtask(app, new_name, action);
                }
                _ => actions::apply_rename(app, new_name, action),
            }
            app.mode = app.last_window.clone();
        } else {
            app.mode = app.last_window.clone();
        }
    } else {
        app.mode = AppMode::Rename(popup, action);
    }
}
