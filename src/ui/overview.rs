use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect },
    widgets::{ Block, BorderType, Borders, List, ListItem, Paragraph },
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, notebook_names: &[String], current_entries: &[String]) {
    // 1. Define the split
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .spacing(2)
        .split(area);

    // 2. Left block
    // List of 'notebooks'
    let notebooks_block = Block::default()
        .title("Notebooks")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let notebooks: Vec<ListItem> = notebook_names
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();
    let notebook_list = List::new(notebooks).block(notebooks_block);
    f.render_widget(notebook_list, chunks[0]);

    // 3. Right block
    // Entries (preview)
    let preview_block = Block::default()
        .title("Preview")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let display_text = current_entries
        .iter()
        .map(|e| format!("• {}", e))
        .collect::<Vec<String>>()
        .join("\n");
    let preview_text = Paragraph::new(display_text).block(preview_block);
    f.render_widget(preview_text, chunks[1]);
}
