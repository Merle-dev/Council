use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Cell, List, ListState, Paragraph, Row, Table, Widget},
};

use crate::{AppState, CouncilListItem, app_state::CreateNewEntry};

pub fn run(terminal: &mut DefaultTerminal, state: &mut AppState) -> Result<()> {
    terminal.draw(|frame| state.draw(frame))?;
    while !state.exit {
        event::read()?
            .as_key_event()
            .map(|key| state.compute_key(key));

        match state.new_item_input.is_some() {
            true => terminal.show_cursor()?,
            false => terminal.hide_cursor()?,
        };
        terminal.draw(|frame| state.draw(frame))?;
    }
    Ok(())
}

impl AppState {
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 4), Constraint::Fill(1)])
            .split(frame.area());

        let list = List::new(
            self.items
                .iter()
                .map(|item| item.name.clone())
                .collect::<Vec<String>>(),
        )
        .direction(ratatui::widgets::ListDirection::TopToBottom)
        .highlight_style(Style {
            fg: Some(ratatui::style::Color::LightBlue),
            add_modifier: Modifier::BOLD,
            ..Default::default()
        })
        .block(Block::bordered());

        frame.render_stateful_widget(
            list,
            layout[0],
            &mut ListState::default().with_selected(Some(self.list_index)),
        );

        if let Some(council_item) = self.items.get(self.list_index) {
            let block = Block::bordered().title(council_item.name.clone());
            frame.render_widget(council_item, block.inner(layout[1]));
            frame.render_widget(block, layout[1]);
        } else {
            frame.render_widget(Block::bordered(), layout[1]);
        }
        if let Some(delete_entry) = self.confirm_deletion_menu {
            let name = self
                .items
                .get(delete_entry)
                .map(|item| item.name.clone())
                .unwrap_or_default();
            let text = format!("Sure to delete: {name} (y) | (n)");
            let center_area = frame.area().centered(
                Constraint::Length(text.len() as u16 + 2),
                Constraint::Length(3),
            );
            frame.render_widget(
                Paragraph::new(text).block(
                    Block::bordered()
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .light_red(),
                ),
                center_area,
            );
        } else if let Some(create_new_entry) = &self.new_item_input {
            let center_area = frame
                .area()
                .centered(Constraint::Ratio(1, 2), Constraint::Ratio(1, 2));
            let cursor_pos = create_new_entry.calculate_cursor_pos(center_area);
            frame.render_widget(create_new_entry.clone(), center_area);
            frame.set_cursor_position(cursor_pos);
        }
    }
}

pub const PASSWORD_GEN_SHOW_LENGTH: u16 = 2;

impl Widget for &CouncilListItem {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let proto_rows = self.info.iter().fold(
            vec![
                ["*".into(), "Name:".into(), self.name.clone()],
                ["*".into(), "Password:".into(), self.password.clone()],
            ],
            |mut acc, (name, data)| {
                acc.push(["*".into(), name.clone(), data.clone()]);
                acc
            },
        );
        let widths = [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ];

        let rows = proto_rows.into_iter().enumerate().fold(
            vec![],
            |mut acc, (idx, [star, name, data])| {
                let data_cell = if self.selected_info == idx {
                    Cell::new(data).style(Style::new().light_yellow().bold())
                } else {
                    Cell::new(data)
                };
                acc.push(Row::new([Cell::new(star), Cell::new(name), data_cell]));
                acc
            },
        );

        Table::new(rows, widths).render(area, buf);
    }
}

impl Widget for CreateNewEntry {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().light_yellow())
            .title("New Item {:?}");

        let pw_row = if self.generate_phase {
            Row::new([
                "*".into(),
                "Password Range:".into(),
                self.item.password_gen_rules[self.calculate_password_gen_range(area)].to_string(),
            ])
        } else {
            Row::new(["*".into(), "Password:".into(), self.item.password.clone()])
        };
        let rows = [
            vec![
                Row::new(["*".into(), "Name: ".into(), self.item.name.clone()]),
                pw_row,
            ],
            self.item
                .info
                .iter()
                .map(|(name, data)| Row::new(["*".into(), name.clone(), data.clone()]))
                .collect(),
        ]
        .concat();

        let widths = [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ];

        Table::new(rows, widths).block(block).render(area, buf);
    }
}
