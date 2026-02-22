mod ui;
mod models;
mod app;

use std::time::Duration;

use ratatui::DefaultTerminal;
use crossterm::event::{ self, Event };

use crate::models::notebook::Notebook;
use crate::models::task::Task;
use crate::app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let mut app = App::new(generate_dummy_data());
    let result = run(&mut terminal, &mut app);

    ratatui::restore();

    result?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> color_eyre::Result<()> {
    loop {
        terminal.draw(|frame| { app.render(frame) })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(key);
            }
        }

        if app.should_quit {
            break Ok(());
        }
    }
}

use crate::models::subtask::Subtask;

fn generate_dummy_data() -> Vec<Notebook> {
    // Dummy data for overview window
    // 20 notebooks, 10 tasks each, 5 subtasks each
    let mut dummy_notebooks = Vec::new();
    for n in 1..=20 {
        let mut tasks = Vec::new();
        for t in 1..=10 {
            let mut subtasks = Vec::new();
            for s in 1..=5 {
                subtasks.push(Subtask {
                    name: format!("Subtask {} (Task {}, Notebook {})", s, t, n),
                    is_done: s % 2 == 0, // Toggle some as done
                });
            }
            
            let mut task = Task {
                name: format!("Task {} (Notebook {})", t, n),
                description: format!("This is the detailed description for Task {}. It has several subtasks to track progress.", t),
                completion: 0.0,
                is_done: false,
                subtasks,
            };
            
            // Sync initial state
            task.recalculate_completion();
            
            tasks.push(task);
        }
        dummy_notebooks.push(Notebook {
            name: format!("Notebook {}", n),
            description: format!("This is the description for Notebook {}. It contains a list of tasks for testing.", n),
            tasks,
        });
    }

    dummy_notebooks
}
