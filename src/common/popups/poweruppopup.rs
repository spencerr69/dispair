use rand::seq::SliceRandom;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Cell, Clear, Row, Table, TableState},
};

use crate::{
    KeyCode, KeyEvent,
    common::{popups::popup_area, powerup::DynPowerup, weapon::WeaponWrapper},
};

pub struct PowerupPopup {
    powerup_choices: Vec<DynPowerup>,
    selection_state: TableState,
    pub weapons: Vec<WeaponWrapper>,
    pub finished: bool,
}

impl PowerupPopup {
    pub fn new(current_weapons: &Vec<WeaponWrapper>) -> Self {
        let mut choices = Vec::new();

        current_weapons.iter().for_each(|weapon| {
            let next_upgrade = weapon.get_inner().get_next_upgrade(1);
            if let Some(next_upgrade) = next_upgrade {
                choices.push(next_upgrade);
            }
        });

        choices.shuffle(&mut rand::rng());

        let mut selection_state = TableState::new();

        if !choices.is_empty() {
            selection_state.select_first();
        }

        Self {
            finished: false,
            weapons: current_weapons.clone(),
            selection_state,
            powerup_choices: choices,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('d') | KeyCode::Right => self.selection_state.select_next_column(),
            KeyCode::Char('a') | KeyCode::Left => self.selection_state.select_previous_column(),
            KeyCode::Enter | KeyCode::Char(' ') => self.select_current(),
            _ => {}
        }
    }

    pub fn select_current(&mut self) {
        if self.powerup_choices.is_empty() {
            self.finished = true;
            return;
        }

        let maybe_selected_index = self.selection_state.selected_cell();
        if let Some((_, col)) = maybe_selected_index {
            if col >= self.powerup_choices.len() {
                return;
            }
            let selected_powerup = &self.powerup_choices[col];

            let mut new_weapons = self.weapons.clone();

            new_weapons.iter_mut().for_each(|weapon| {
                if weapon.get_inner().get_name() == selected_powerup.get_name() {
                    weapon.get_inner_mut().upgrade_self(selected_powerup);
                }
            });

            self.weapons = new_weapons;
            self.finished = true;
        }
    }

    pub fn render_choices(&mut self, frame: &mut Frame, rect: Rect) {
        let mut texts: Vec<Cell> = Vec::new();

        for choice in &self.powerup_choices {
            let widths = [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ];

            let area_width = Layout::horizontal(widths).areas::<3>(rect)[0];

            let raw_title = choice.get_name();
            let title = String::from(raw_title);

            let raw_desc = choice.get_desc();
            let desc = String::from(raw_desc);

            let curr_level = choice.get_current_level();
            let new_level = choice.get_new_level();
            let amount = format!("Level {} -> {}", curr_level, new_level);

            let powerup_text =
                String::from_iter(vec![title, "\n".into(), desc, "\n".into(), amount]);

            let wrapped = textwrap::wrap(powerup_text.as_str(), area_width.width as usize);

            let yass: Vec<Line> = wrapped
                .iter()
                .map(|line| Line::from(line.to_string()))
                .collect();

            let text = Text::from(yass).centered();

            let text = Cell::from(text);

            texts.push(text);
        }

        let row = [Row::new(texts).height(rect.height)];

        let table = Table::new(
            row,
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ],
        );

        let table = table.cell_highlight_style(Style::default().fg(Color::Yellow));

        frame.render_stateful_widget(table, rect, &mut self.selection_state);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = popup_area(frame.area(), 80, 60);

        let popup = Block::bordered()
            .border_set(border::PLAIN)
            .title(" Powerup Choice ")
            .title_alignment(ratatui::layout::Alignment::Center);

        let inner_area = popup.inner(area);

        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
        self.render_choices(frame, inner_area);
    }
}
