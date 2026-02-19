mod ui;
mod models;

use std::time::Duration;

use crossterm::event::{ self, Event, KeyCode, KeyEventKind };
use ratatui::DefaultTerminal;

use crate::models::notebook::Notebook;
use crate::models::task::Task;
use crate::ui::overview::{ self, Overview };

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let result = app(&mut terminal);

    ratatui::restore();

    result?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
    // Dummy data for overview window
    // 20 notebooks, 10 tasks each
    let mut dummy_notebooks = Vec::new();
    for n in 1..=20 {
        let mut tasks = Vec::new();
        for t in 1..=10 {
            tasks.push(Task {
                name: format!("Task {} (Notebook {})", t, n),
                description: String::from(""),
                completion: 0.0,
                is_done: false,
                subtasks: Vec::new(),
            });
        }
        dummy_notebooks.push(Notebook {
            name: format!("Notebook {}", n),
            tasks,
        });
    }

    let mut overview = Overview::new(dummy_notebooks);

    loop {
        terminal.draw(|frame| { overview.render(frame, frame.area()) })?;

        if handle_key(&mut overview)? {
            break Ok(());
        }
    }
}

fn handle_key(overview: &mut Overview) -> color_eyre::Result<bool> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Global keys
                if key.code == KeyCode::Char('q') {
                    return Ok(true);
                }

                // Component keys
                overview.handle_key(key);
            }
        }
    }
    Ok(false)
}
