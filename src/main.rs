mod app;
mod models;
mod storage;
mod ui;

use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use crate::app::App;
use crate::storage::paths::FileSystem;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let fs = FileSystem::new()?;
    let mut app = App::new(fs);
    let result = run(&mut terminal, &mut app);

    ratatui::restore();
    result?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> color_eyre::Result<()> {
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    app.handle_input(key);
                }
            }
        }

        if app.should_quit {
            break Ok(());
        }
    }
}
