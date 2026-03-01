use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, palette::tailwind},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::app::AppMode;

pub struct HelpPopup;

impl HelpPopup {
    pub fn render(f: &mut Frame, area: Rect, context: &AppMode) {
        // Anchor to bottom-left
        let width = 50;
        let height = 11;
        let popup_area = Rect {
            x: area.x + 1,
            y: area.height.saturating_sub(height + 1),
            width: width.min(area.width.saturating_sub(2)),
            height: height.min(area.height.saturating_sub(2)),
        };

        f.render_widget(Clear, popup_area);

        let block = Block::bordered()
            .title("Help ")
            .title_alignment(Alignment::Left)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(tailwind::EMERALD.c600));

        let mut help_lines = Vec::new();
        
        match context {
            AppMode::Overview => {
                help_lines.push(String::from(" [a] : New Notebook     [e] : Edit Notebook"));
                help_lines.push(String::from(" [r] : Rename Notebook  [d] : Delete Notebook"));
                help_lines.push(String::from(" [Enter]: Open Notebook [q] : Quit"));
            }
            AppMode::NotebookDetail => {
                help_lines.push(String::from(" [A/I]: Task Add/Ins    [a/i]: Subtask Add/Ins"));
                help_lines.push(String::from(" [r]  : Rename Task     [e]  : Rename Subtask"));
                help_lines.push(String::from(" [D]  : Delete Task     [d]  : Delete Subtask"));
                help_lines.push(String::from(" [X]  : Toggle Task     [x]  : Toggle Subtask"));
                help_lines.push(String::from(" [E]  : Full Inspector  [Esc]: Back"));
            }
            _ => {
                help_lines.push(String::from(" [Tab]: Next Field      [S-Tab]: Prev Field"));
                help_lines.push(String::from(" [Enter]: Add Item      [Alt-Enter]: Save All"));
                help_lines.push(String::from(" [Esc]: Cancel Changes"));
            }
        }
        
        help_lines.push(String::new());
        help_lines.push(String::from(" Press any key to close help "));

        let content = Paragraph::new(help_lines.join("
"))
            .block(block)
            .style(Style::default().fg(tailwind::WHITE));

        f.render_widget(content, popup_area);
    }
}
