use std::default;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::palette::tailwind,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

#[derive(PartialEq, Clone)]
pub enum InspectMode {
    View,
    Edit,
    Add,
}

#[derive(PartialEq, Clone)]
pub enum InspectField {
    Title,
    Description,
    Contents,
}

#[derive(PartialEq, Clone)]
pub struct Inspector {
    pub mode: InspectMode,
    pub title_input: String,
    pub desc_input: String,
    pub list_items: Vec<String>,
    pub list_label: String,
    pub focused_field: InspectField,
}

impl Inspector {
    pub fn new(
        mode: InspectMode,
        title_input: String,
        desc_input: String,
        list_items: Vec<String>,
        list_label: String,
    ) -> Self {
        Self {
            mode,
            title_input,
            desc_input,
            list_items,
            list_label,
            focused_field: InspectField::Title,
        }
    }
}

impl Inspector {
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Title (Compact)
                Constraint::Percentage(30), // Description
                Constraint::Fill(1),        // Contents (Rest of the space)
            ])
            .split(area);

        // Colors
        let focused_color = tailwind::ROSE.c500;
        let default_color = tailwind::WHITE;

        // -- Title --
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(match &self.focused_field {
                InspectField::Title => focused_color,
                _ => default_color,
            })
            .title("Title")
            .title_alignment(Alignment::Left);

        let title_block_inner = title_block.inner(chunks[0]);
        f.render_widget(title_block, chunks[0]);

        let title_display =
            if self.focused_field == InspectField::Title && self.mode != InspectMode::View {
                format!("{}█", self.title_input)
            } else {
                self.title_input.clone()
            };
        f.render_widget(Paragraph::new(title_display), title_block_inner);

        // -- Description --
        let description_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(match &self.focused_field {
                InspectField::Description => focused_color,
                _ => default_color,
            })
            .title("Description")
            .title_alignment(Alignment::Left);

        let description_block_inner = description_block.inner(chunks[1]);
        f.render_widget(description_block, chunks[1]);

        let desc_display =
            if self.focused_field == InspectField::Description && self.mode != InspectMode::View {
                format!("{}█", self.desc_input)
            } else {
                self.desc_input.clone()
            };
        let desc_paragraph = Paragraph::new(desc_display).wrap(Wrap { trim: true });
        f.render_widget(desc_paragraph, description_block_inner);

        // -- Contents --
        let contents_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(match &self.focused_field {
                InspectField::Contents => focused_color,
                _ => default_color,
            })
            .title(self.list_label.as_str())
            .title_alignment(Alignment::Left);

        let contents_block_inner = contents_block.inner(chunks[2]);
        f.render_widget(contents_block, chunks[2]);

        let items: Vec<ListItem> = self
            .list_items
            .iter()
            .map(|name| ListItem::new(format!(" • {}", name)))
            .collect();
        f.render_widget(List::new(items), contents_block_inner);
    }
}
