use std::str::FromStr;

use rand::seq::SliceRandom;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Cell, Clear, Row, Table, TableState},
};
use strum::IntoEnumIterator;

use crate::{
    common::{
        charms::CharmWrapper,
        popups::popup_area,
        powerup::{DynPowerup, PowerupTypes, PowerupUpgrade},
        stats::WeaponStats,
        weapons::WeaponWrapper,
    },
    target_types::{KeyCode, KeyEvent},
};

pub struct PowerupPopup {
    powerup_choices: Vec<DynPowerup>,
    selection_state: TableState,
    pub weapons: Vec<WeaponWrapper>,
    pub charms: Vec<CharmWrapper>,
    pub base_weapon_stats: WeaponStats,
    pub finished: bool,
}

impl PowerupPopup {
    pub fn new(
        current_weapons: &[WeaponWrapper],
        current_charms: &[CharmWrapper],
        weapon_stats: WeaponStats,
    ) -> Self {
        let mut choices = Vec::new();

        WeaponWrapper::iter().for_each(|weapon_wrapper| {
            if let Some(weapon) = current_weapons.iter().find(|w| *w == &weapon_wrapper) {
                let next_upgrade = weapon.get_inner().get_next_upgrade(1);
                if let Some(next_upgrade) = next_upgrade {
                    choices.push(next_upgrade);
                }
            } else if current_weapons.len() < 3 {
                let weapon = weapon_wrapper;
                let powerup = PowerupUpgrade::init_weapon(weapon);
                choices.push(Box::new(powerup));
            }
        });

        CharmWrapper::iter().for_each(|charm_wrapper| {
            if let Some(charm) = current_charms.iter().find(|c| *c == &charm_wrapper) {
                let next_upgrade = charm.get_inner().get_next_upgrade(1);
                if let Some(next_upgrade) = next_upgrade {
                    choices.push(next_upgrade);
                }
            } else if current_charms.len() < 3 {
                let charm = charm_wrapper;
                let powerup = PowerupUpgrade::init_charm(charm);
                choices.push(Box::new(powerup));
            }
        });

        choices.shuffle(&mut rand::rng());

        let _ = choices.split_off(3.min(choices.len()));

        let mut selection_state = TableState::new();

        if !choices.is_empty() {
            selection_state.select_first();
        }

        Self {
            finished: false,
            weapons: Vec::from(current_weapons),
            charms: Vec::from(current_charms),
            selection_state,
            powerup_choices: choices,
            base_weapon_stats: weapon_stats,
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

            match selected_powerup.get_powerup_type() {
                PowerupTypes::Weapon => {
                    let mut new_weapons = self.weapons.clone();
                    new_weapons.iter_mut().for_each(|weapon| {
                        if weapon.get_inner().get_name().to_uppercase()
                            == selected_powerup.get_name().to_uppercase()
                        {
                            weapon.get_inner_mut().upgrade_self(selected_powerup);
                        }
                    });

                    if !new_weapons.iter().any(|weapon| {
                        weapon.get_inner().get_name().to_uppercase()
                            == selected_powerup.get_name().to_uppercase()
                    }) && let Ok(mut new_weapon) =
                        WeaponWrapper::from_str(selected_powerup.get_name().to_uppercase().as_str())
                    {
                        new_weapon.populate_inner(self.base_weapon_stats.clone());
                        new_weapons.push(new_weapon)
                    }

                    self.weapons = new_weapons;
                }

                PowerupTypes::Charm => {
                    let mut new_charms = self.charms.clone();
                    new_charms.iter_mut().for_each(|charm| {
                        if charm.get_inner().get_name().to_uppercase()
                            == selected_powerup.get_name().to_uppercase()
                        {
                            charm.get_inner_mut().upgrade_self(selected_powerup);
                        }
                    });

                    if !new_charms.iter().any(|charm| {
                        charm.get_inner().get_name().to_uppercase()
                            == selected_powerup.get_name().to_uppercase()
                    }) && let Ok(mut new_charm) =
                        CharmWrapper::from_str(selected_powerup.get_name().to_uppercase().as_str())
                    {
                        new_charm.populate_inner();
                        new_charms.push(new_charm)
                    }
                    self.charms = new_charms;
                }
            };

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

        let table =
            table.cell_highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black));

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
