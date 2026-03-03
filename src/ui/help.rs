use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Modifier},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::app::AppMode;
use crate::ui::theme::theme;

pub struct HelpPopup;

impl HelpPopup {
    pub fn render(f: &mut Frame, area: Rect, context: &AppMode) {
        let width = 60;
        let height = 18;
        let popup_area = Self::centered_rect(width, height, area);

        f.render_widget(Clear, popup_area);

        let block = Block::bordered()
            .title(" Help - Keybindings ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme().title_secondary));

        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Spacer
                Constraint::Fill(1),   // Dynamic Sections
                Constraint::Length(1), // Footer
            ])
            .split(inner_area);

        // Sections Container
        let section_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45), // Top Section
                Constraint::Percentage(10), // Gap
                Constraint::Percentage(45), // Bottom Section
            ])
            .split(chunks[1]);

        match context {
            AppMode::Overview => {
                Self::render_section(f, section_chunks[0], "Notebooks", 
                    &["[a]: New Notebook", "[r]: Rename Notebook", "[Enter]: Open Notebook"],
                    &["[e]: Edit Details", "[d]: Delete Notebook"]
                );
                Self::render_section(f, section_chunks[2], "Navigation", 
                    &["[h/l]: Move left/right", "[q]: Quit application"],
                    &["[j/k]: Move up/down", "[Alt-T]: Cycle theme", "[?]: Toggle help"]
                );
            }
            AppMode::NotebookDetail => {
                Self::render_section(f, section_chunks[0], "Tasks", 
                    &["[A/I]: Add after/before", "[D]: Delete task", "[E/Enter]: Inspector"],
                    &["[r]: Rename task", "[X]: Toggle completion", "[S-H/L]: Move Task"]
                );
                Self::render_section(f, section_chunks[2], "Subtasks", 
                    &["[a/i]: Add after/before", "[d]: Delete subtask"],
                    &["[e]: Rename subtask", "[x]: Toggle status", "[S-J/K]: Move Subtask"]
                );
            }
            _ => {
                Self::render_section(f, section_chunks[0], "Fields", 
                    &["[Tab]: Next field", "[Enter]: Next / Add item"],
                    &["[S-Tab]: Previous field"]
                );
                Self::render_section(f, section_chunks[2], "Actions", 
                    &["[Ctrl-S]: Save and exit"],
                    &["[Esc]: Discard / Back"]
                );
            }
        }

        // Footer
        f.render_widget(
            Paragraph::new("Press any key to close")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme().title_secondary).add_modifier(Modifier::ITALIC)),
            chunks[2],
        );
    }

    fn render_section(f: &mut Frame, area: Rect, title: &str, left: &[&str], right: &[&str]) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(0),    // Columns
            ])
            .split(area);

        // Header
        f.render_widget(
            Paragraph::new(format!("[ {} ]", title))
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme().title_secondary).add_modifier(Modifier::BOLD)),
            chunks[0],
        );

        // Columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(5),  // Margin
                Constraint::Percentage(45), // Left Column
                Constraint::Percentage(45), // Right Column
                Constraint::Percentage(5),  // Margin
            ])
            .split(chunks[1]);

        f.render_widget(Paragraph::new(left.join("\n")), columns[1]);
        f.render_widget(Paragraph::new(right.join("\n")), columns[2]);
    }

    fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(height),
                Constraint::Min(0),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(width),
                Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    }
}
