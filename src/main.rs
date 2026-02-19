use std::time::Duration;

use crossterm::event::{ self, Event, KeyCode, KeyEventKind };
use ratatui::DefaultTerminal;

mod ui;

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
    let notebooks: Vec<String> = vec!["Work".to_string(), "Personal".to_string()];
    let entries: Vec<String> = vec!["Task 1".to_string(), "Task 2".to_string()];

    loop {
        terminal.draw( |frame| {
            ui::overview::render(
                frame,
                frame.area(),
                &notebooks,
                &entries
            )
        })?;
        if quit()? {
            break Ok(());
        }
    }
}

fn quit() -> color_eyre::Result<bool> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}