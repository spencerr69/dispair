use crate::common::PlayerStateRef;
use crate::common::charms::CharmWrapper;
use crate::common::enemies::enemywrangler::EnemyWrangler;
use crate::common::utils::trim_string;
use crate::common::weapons::{Weapon, WeaponWrapper, get_strongest_weapon};
use crate::common::widgets::inviconwidget::InvIconWidget;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType};

pub struct StatsWidget<'a> {
    pub player_state: PlayerStateRef,
    pub enemy_wrangler: &'a EnemyWrangler,
    pub weapons: &'a Vec<WeaponWrapper>,
    pub charms: &'a Vec<CharmWrapper>,
}

impl<'a> StatsWidget<'a> {
    pub fn new(
        player_state: PlayerStateRef,
        enemy_wrangler: &'a EnemyWrangler,
        weapons: &'a Vec<WeaponWrapper>,
        charms: &'a Vec<CharmWrapper>,
    ) -> Self {
        Self {
            player_state,
            enemy_wrangler,
            weapons,
            charms,
        }
    }

    #[must_use]
    pub fn get_stat_vecs(&self) -> (Vec<Line<'_>>, Vec<Line<'_>>) {
        // Extract stats and their corresponding labels into separate vectors
        let mut stat_labels = Vec::new();
        let mut stat_values = Vec::new();

        let player_stats = &self.player_state.borrow().stats.player_stats;
        let game_stats = &self.player_state.borrow().stats.game_stats;
        let weapon_stats = &self.player_state.borrow().stats.weapon_stats;

        let best_weapon = get_strongest_weapon(self.weapons)
            .map(|w| w.get_damage())
            .unwrap_or_default();

        stat_labels.push(Line::raw("damage_boost").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!("+{}", weapon_stats.damage_flat_boost),
                5,
            ))
            .right_aligned(),
        );

        stat_labels.push(Line::raw("damage_mult").left_aligned());
        stat_values.push(
            Line::raw(trim_string(format!("x{}", player_stats.damage_mult), 5)).right_aligned(),
        );
        stat_labels.push(Line::raw("max_damage").left_aligned());
        stat_values.push(Line::raw(trim_string(format!("{best_weapon}"), 5)).right_aligned());

        stat_labels.push(Line::from(""));
        stat_values.push(Line::from(""));

        stat_labels.push(Line::raw("base_health").left_aligned());
        stat_values.push(
            Line::raw(trim_string(format!("{}", player_stats.base_health), 5)).right_aligned(),
        );

        stat_labels.push(Line::raw("health_mult").left_aligned());
        stat_values.push(
            Line::raw(trim_string(format!("x{}", player_stats.health_mult), 5)).right_aligned(),
        );

        stat_labels.push(Line::from(""));
        stat_values.push(Line::from(""));

        stat_labels.push(Line::raw("attack_speed_mult").left_aligned());
        stat_values.push(
            Line::raw(trim_string(format!("x{}", game_stats.attack_speed_mult), 5)).right_aligned(),
        );
        stat_labels.push(Line::raw("movement_speed_mult").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!("x{}", player_stats.movement_speed_mult),
                5,
            ))
            .right_aligned(),
        );

        stat_labels.push(Line::from(""));
        stat_values.push(Line::from(""));

        stat_labels.push(Line::from("enemy_health").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!("{}", self.enemy_wrangler.enemy_health),
                5,
            ))
            .right_aligned(),
        );
        stat_labels.push(Line::from("enemy_damage").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!("{}", self.enemy_wrangler.enemy_damage),
                5,
            ))
            .right_aligned(),
        );
        stat_labels.push(Line::from("enemy_spawn_speed").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!("x{}", self.enemy_wrangler.get_spawn_multiplier()),
                5,
            ))
            .right_aligned(),
        );
        stat_labels.push(Line::from("enemy_move_speed").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!(
                    "{}%",
                    (self.enemy_wrangler.enemy_move_ticks as i32 - 100).abs()
                ),
                5,
            ))
            .right_aligned(),
        );

        stat_labels.push(Line::from(""));
        stat_values.push(Line::from(""));

        stat_labels.push(Line::from("enemy_count").left_aligned());
        stat_values.push(
            Line::raw(trim_string(
                format!("{}", self.enemy_wrangler.enemies.borrow().len()),
                5,
            ))
            .right_aligned(),
        );

        (stat_labels, stat_values)
    }
}

impl Widget for StatsWidget<'_> {
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
        let left_text = Text::from(stat_labels);
        let right_text = Text::from(stat_values);

        Self::create_icons(buf, bottom, self.weapons, self.charms);

        left_text.render(left, buf);
        right_text.render(right, buf);
        block.render(area, buf);
    }
}

impl StatsWidget<'_> {
    fn create_icons(
        buf: &mut Buffer,
        area: Rect,
        weapons: &Vec<WeaponWrapper>,
        charms: &Vec<CharmWrapper>,
    ) {
        let [icons_top, icons_bottom] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);

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

        for (i, charm) in charms.iter().enumerate() {
            let icon = InvIconWidget::new(charm.get_inner());
            icon.render(icons_top[i], buf)
        }
        for (i, weapon) in weapons.iter().enumerate() {
            let icon = InvIconWidget::new(weapon.get_inner());
            icon.render(icons_bottom[i], buf)
        }
    }
}
