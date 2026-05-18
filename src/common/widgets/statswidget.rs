use crate::common::upgrades::upgrade::PlayerState;
use crate::common::weapons::WeaponWrapper::Flash;
use crate::common::weapons::{WeaponWrapper, flash};
use crate::common::widgets::inviconwidget::InvIconWidget;
use crate::common::{PlayerStateRef, charms};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct StatsWidget {
    pub player_state: PlayerStateRef,
}

impl StatsWidget {
    pub fn new(player_state: PlayerStateRef) -> Self {
        Self { player_state }
    }

    pub fn get_stat_vecs(&self) -> (Vec<Line<'_>>, Vec<Line<'_>>) {
        // Extract stats and their corresponding labels into separate vectors
        let mut stat_labels = Vec::new();
        let mut stat_values = Vec::new();

        let player_stats = &self.player_state.borrow().stats.player_stats;
        let game_stats = &self.player_state.borrow().stats.game_stats;
        let weapon_stats = &self.player_state.borrow().stats.weapon_stats;

        stat_labels.push(Line::raw("damage_raw").left_aligned());
        stat_values.push(Line::raw(format!("+{}", weapon_stats.damage_flat_boost)).right_aligned());

        stat_labels.push(Line::raw("damage_mult").left_aligned());
        stat_values.push(Line::raw(format!("x{}", player_stats.damage_mult)).right_aligned());

        stat_labels.push(Line::from(""));
        stat_values.push(Line::from(""));

        stat_labels.push(Line::raw("base_health").left_aligned());
        stat_values.push(Line::raw(format!("{}", player_stats.base_health)).right_aligned());

        stat_labels.push(Line::raw("health_mult").left_aligned());
        stat_values.push(Line::raw(format!("x{}", player_stats.health_mult)).right_aligned());

        stat_labels.push(Line::from(""));
        stat_values.push(Line::from(""));

        stat_labels.push(Line::raw("attack_speed_mult").left_aligned());
        stat_values.push(Line::raw(format!("x{}", game_stats.attack_speed_mult)).right_aligned());
        stat_labels.push(Line::raw("movement_speed_mult").left_aligned());
        stat_values
            .push(Line::raw(format!("x{}", player_stats.movement_speed_mult)).right_aligned());

        (stat_labels, stat_values)
    }
}

impl Widget for StatsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Logic to render the stats widget

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().dark_gray())
            .title_top("Stats");
        let inner_area = block.inner(area);

        let [top, bottom] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(6)]).areas(inner_area);

        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(5)]).areas(top);

        let (stat_labels, stat_values) = self.get_stat_vecs();

        let [icons_top, icons_bottom] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(bottom);

        let icons_top: [Rect; 5] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .areas(icons_top);
        let icons_bottom: [Rect; 5] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .areas(icons_bottom);

        let left_text = Text::from(stat_labels);
        let right_text = Text::from(stat_values);

        let test_powerup = charms::attack_speed::CharmAttackSpeed {
            stat_boost: 1.0,
            level: 1,
            player_state: Rc::new(RefCell::new(PlayerState::default())),
        };

        let test_icon = InvIconWidget::new(&test_powerup);

        for a in icons_top {
            test_icon.clone().render(a, buf);
        }

        for a in icons_bottom {
            test_icon.clone().render(a, buf);
        }

        left_text.render(left, buf);
        right_text.render(right, buf);
        block.render(area, buf);
    }
}
