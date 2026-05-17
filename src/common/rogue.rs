//! This module implements the core game logic for the roguelike.
//! It manages game state, character movement, enemy behavior, and rendering.

use crate::common::character::Renderable;
use crate::common::enemies::enemy::{Enemy, EnemyDrops};
use crate::common::enemies::enemywrangler::EnemyWrangler;
use crate::common::entities::EntityCharacters;
use crate::common::pickups::pickupwrangler::PickupWrangler;
use crate::common::render::{flatten_to_span, get_camera_area, spans_to_text};
use crate::common::upgrades::upgrade::CurrentUpgradesTrait;
use crate::common::utils::{center, move_entity, per_sec_to_tick_count};
use crate::common::{Goto, PlayerStateRef, Viewable};
use crate::{
    common::{
        TICK_RATE,
        character::{Character, Damageable, Movable},
        coords::{Direction, Position, SquareArea},
        effects::DamageEffect,
        level::Level,
        popups::{carnagereport::CarnageReport, poweruppopup::PowerupPopup},
        timescaler::TimeScaler,
        upgrades::upgrade::PlayerState,
    },
    prelude::{Duration, Instant, KeyCode, KeyEvent},
};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Gauge, Paragraph},
};
use std::cell::RefCell;
use std::rc::Rc;

pub type Layer = Vec<Vec<EntityCharacters>>;

pub enum GameState {
    Paused,
    GameOver,
    Exit,
    Play,
}

/// Represents the main game state and logic.
pub struct Rogue {
    /// The player's current state, including stats and inventory.
    pub player_state: PlayerStateRef,
    init_state: PlayerState,

    /// The carnage report, which is displayed at the end of a level.
    pub carnage_report: Option<CarnageReport>,

    pub powerup_popup: Option<PowerupPopup>,

    /// The rendered map text.
    pub map_text: Text<'static>,

    pub character: Character,
    pub layer_base: Layer,
    pub flat_layer: Layer,

    tickcount: u64,

    height: usize,
    width: usize,

    pub enemies: Rc<RefCell<Vec<Enemy>>>,

    enemy_wrangler: EnemyWrangler,

    attack_ticks: u64,

    pub game_state: GameState,

    pub goto: Goto,

    pub active_damage_effects: Vec<DamageEffect>,

    pub pickup_wrangler: PickupWrangler,

    pub level: Level,

    timer: Duration,
    start_time: Instant,

    start_popup: bool,

    timescaler: Rc<RefCell<TimeScaler>>,

    view_area: Rect,
    camera_area: SquareArea,
}

impl Rogue {
    const DEFAULT_ATTACK_P_S: f64 = 1.5;

    #[must_use]
    pub fn new(player_state: &PlayerStateRef) -> Self {
        let init_player_state = player_state.borrow().clone();

        let width = init_player_state.stats.game_stats.width;
        let height = init_player_state.stats.game_stats.height;

        let mut base: Layer = Vec::new();

        let mut rng = rand::rng();

        for _ in 0..height {
            let mut baseline = Vec::new();
            for _ in 0..width {
                let choice = rng.random_range(0..=1);
                match choice {
                    0 => baseline.push(EntityCharacters::Background1),
                    _ => baseline.push(EntityCharacters::Background2),
                }
            }
            base.push(baseline);
        }

        let attack_ticks = per_sec_to_tick_count(Self::DEFAULT_ATTACK_P_S);

        let start_time = Instant::now();
        let timer = Duration::from_secs(init_player_state.stats.game_stats.timer);

        let timescaler = Rc::new(RefCell::new(TimeScaler::now()));
        timescaler
            .borrow_mut()
            .offset_start_time(init_player_state.stats.game_stats.time_offset);

        let enemies = Rc::new(RefCell::new(Vec::new()));

        let level = Level::new();

        let pickup_wrangler = PickupWrangler::new(player_state.clone());

        let mut game = Rogue {
            goto: Goto::Game,

            player_state: player_state.clone(),
            init_state: init_player_state,
            character: Character::new(player_state.clone()),
            layer_base: base.clone(),
            flat_layer: base,
            height,
            width,
            attack_ticks,

            enemy_wrangler: EnemyWrangler::new(
                player_state.clone(),
                timescaler.clone(),
                enemies.clone(),
            ),

            map_text: Text::from(""),
            start_popup: false,

            game_state: GameState::Play,

            carnage_report: None,
            powerup_popup: None,

            level,

            tickcount: 0,
            enemies,
            pickup_wrangler,
            active_damage_effects: vec![],
            start_time,
            timer,
            timescaler,

            //IDGAF !!! there shouldn't be any cases where values get truncated here
            #[allow(clippy::cast_possible_truncation)]
            view_area: Rect::new(0, 0, width as u16, height as u16),
            #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
            camera_area: SquareArea::new(Position(0, 0), Position(width as i32, height as i32)),
        };

        game.init_character();

        game.update_stats_with_charms();
        game.update_stats();

        if game.player_state.borrow().upgrade_owned("53") {
            game.pickup_wrangler.spawn_orb(&game.layer_base);
        }

        game
    }

    pub fn on_tick(&mut self) {
        self.handle_popup();

        match self.game_state {
            GameState::Paused | GameState::Exit => {}
            GameState::GameOver => {
                self.carnage_report = Some(CarnageReport::new(
                    self.init_state.clone(),
                    self.player_state.borrow().clone(),
                ));
            }
            GameState::Play => {
                self.tickcount += 1;

                if self.pickup_wrangler.start_popup {
                    self.start_popup = true;
                    self.pickup_wrangler.start_popup = false;
                }

                if self.start_time.elapsed() >= self.timer {
                    self.game_state = GameState::GameOver;
                    return;
                }

                if !self.character.is_alive() {
                    self.game_state = GameState::GameOver;
                    return;
                }

                if self.level.update().is_some() {
                    self.start_popup = true;
                }

                if self.start_popup {
                    self.generate_popup();
                }

                let char_pos = self.get_character_pos().clone();

                self.pickup_wrangler.on_tick(
                    self.tickcount,
                    &char_pos,
                    &mut self.active_damage_effects,
                );

                let drops = self.enemy_wrangler.on_tick(
                    self.tickcount,
                    &mut self.character,
                    &self.layer_base,
                    &mut self.active_damage_effects,
                );

                for drop in drops {
                    self.consume_drops(&drop);
                }

                if self.tickcount.is_multiple_of(TICK_RATE.floor() as u64) {
                    self.scale();
                }

                if self.tickcount.is_multiple_of(self.attack_ticks) {
                    let (damage_areas, mut damage_effects) = self
                        .character
                        .attack(&self.layer_base, &self.enemies.borrow());
                    for area in damage_areas {
                        area.deal_damage(&mut self.enemies.borrow_mut());
                    }
                    self.active_damage_effects.append(&mut damage_effects);
                }
            }
        }
    }

    fn handle_popup(&mut self) {
        if let Some(powerup_popup) = self.powerup_popup.take() {
            if powerup_popup.finished {
                self.game_state = GameState::Play;
                self.character.weapons = powerup_popup.weapons;
                self.character.charms = powerup_popup.charms;
                self.reset_stats();
                self.update_stats_with_charms();
                self.update_stats();

                self.player_state.borrow_mut().upgrades.set("A", 1);
            } else {
                self.powerup_popup = Some(powerup_popup);
            }
        }
    }

    pub fn consume_drops(&mut self, drops: &EnemyDrops) {
        let mut player_state = self.player_state.borrow_mut();

        player_state.inventory.gold +=
            (drops.gold as f64 * player_state.stats.game_stats.gold_mult) as u128;
        self.level.add_xp(drops.xp);
    }

    pub fn on_frame(&mut self) {
        if let GameState::Play = self.game_state {
            update_effects(&mut self.active_damage_effects);

            self.active_damage_effects = self
                .active_damage_effects
                .clone()
                .into_iter()
                .filter(|effect| !effect.complete)
                .collect();

            self.camera_area =
                get_camera_area(self.view_area, self.get_character_pos(), &self.layer_base);

            let spans = flatten_to_span(&self, Some(self.camera_area.clone()));

            self.map_text = spans_to_text(spans);
        }
    }

    pub fn update_stats(&mut self) {
        self.attack_ticks = per_sec_to_tick_count(Self::DEFAULT_ATTACK_P_S);
        self.attack_ticks = (self.attack_ticks as f64
            / self
                .player_state
                .borrow()
                .stats
                .game_stats
                .attack_speed_mult)
            .ceil() as u64;

        let offset = self.player_state.borrow().stats.game_stats.time_offset;

        self.timescaler.borrow_mut().offset_start_time(offset);
    }

    pub fn generate_popup(&mut self) {
        self.game_state = GameState::Paused;
        self.powerup_popup = Some(PowerupPopup::new(
            &self.character.weapons,
            &self.character.charms,
            self.player_state.borrow().stats.weapon_stats.clone(),
            self.player_state.clone(),
        ));
        self.start_popup = false;
    }

    fn scale(&mut self) -> f64 {
        self.timescaler.borrow_mut().scale()
    }

    pub fn key_event(&mut self, key_event: &KeyEvent) {
        if self.carnage_report.is_some() {
            if key_event.code == KeyCode::Esc {
                self.game_state = GameState::Exit;
                self.goto = Goto::Upgrades;
            }
        } else if let Some(powerup_popup) = &mut self.powerup_popup {
            powerup_popup.handle_key_event(key_event);
        } else {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Down => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::DOWN);
                }
                KeyCode::Char('w') | KeyCode::Up => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::UP);
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::RIGHT);
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::LEFT);
                }
                KeyCode::Esc => self.game_state = GameState::GameOver,
                #[cfg(debug_assertions)]
                KeyCode::Char('u') => self.generate_popup(),
                _ => {}
            }
        }
    }

    pub fn init_character(&mut self) {
        let mut rng = rand::rng();

        let (x, y) = (
            rng.random_range(0..self.width) as i32,
            rng.random_range(0..self.height) as i32,
        );

        self.character.set_pos(Position(x, y));
    }

    pub fn reset_stats(&mut self) {
        self.player_state.borrow_mut().refresh();
    }

    pub fn update_stats_with_charms(&mut self) {
        self.character.charms.iter().for_each(|charm_wrapper| {
            charm_wrapper
                .get_inner()
                .manipulate_stats(&mut self.player_state.borrow_mut().stats);
        });
    }

    /// Returns the character's current position.
    #[must_use]
    pub fn get_character_pos(&self) -> &Position {
        self.character.get_pos()
    }

    pub fn render_game(&mut self, frame: &mut Frame) {
        let timer = self.timer.saturating_sub(self.start_time.elapsed());

        let title = Line::from(" dispair.run ".bold());

        let instructions = Line::from(vec![
            " Health: ".dark_gray(),
            self.character.get_health().to_string().bold(),
            " ".into(),
            " Time: ".dark_gray(),
            timer.as_secs().to_string().bold(),
            " ".into(),
            " Gold: ".dark_gray(),
            self.player_state.borrow().inventory.gold.to_string().into(),
            " ".into(),
        ]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.right_aligned())
            .border_set(border::THICK);

        let mut game_area = block.inner(frame.area());
        frame.render_widget(&block, frame.area());

        if self.player_state.borrow().upgrade_owned("A") {
            let progress_bar_area;

            [progress_bar_area, game_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(game_area);

            let progress_bar = Gauge::default()
                .gauge_style(Style::new().light_blue())
                .percent(self.level.get_progress_percentage());

            frame.render_widget(progress_bar, progress_bar_area);
        }

        self.view_area = game_area;

        let content_area = self.view_area;

        let height = self.map_text.lines.len() as u16;
        let width = self.map_text.lines[0].iter().len() as u16;

        let centered_area = center(content_area, width, height);

        let content = Paragraph::new(self.map_text.clone()).centered();

        frame.render_widget(content, centered_area);

        if let Some(ref mut carnage) = self.carnage_report {
            carnage.render(frame);
        }

        if let Some(ref mut powerup_popup) = self.powerup_popup {
            powerup_popup.render(frame);
        }
    }
}

/// Calculates the camera's visible area based on the player's position and the layer dimensions.

impl Viewable for Rogue {
    fn tick(&mut self) {
        self.on_tick();
    }

    fn frame(&mut self) {
        self.on_frame();
    }

    fn get_goto(&self) -> &Goto {
        &self.goto
    }

    fn render(&mut self, frame: &mut Frame) {
        self.render_game(frame);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        self.key_event(key_event);
    }
}

pub fn update_effects(damage_effects: &mut [DamageEffect]) {
    for effect in damage_effects.iter_mut() {
        effect.update();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Instant;

    use crate::common::render::{flatten_to_span, spans_to_text};
    use crate::common::{rogue::Rogue, upgrades::upgrade::PlayerState};

    #[test]
    fn renderspeed() {
        let mut player_state = PlayerState::default();

        player_state.stats.game_stats.width = 1000;
        player_state.stats.game_stats.height = 1000;

        let mut rogue_game = Rogue::new(&Rc::new(RefCell::new(player_state)));

        rogue_game.on_tick();
        rogue_game.on_frame();

        let start_time = Instant::now();

        let spans = flatten_to_span(&rogue_game, None);
        let _ = spans_to_text(spans);

        let elapsed = start_time.elapsed().as_millis();

        println!("Renderspeed Time taken: {elapsed}");

        #[cfg(not(debug_assertions))]
        assert!(elapsed < 100);
    }

    #[test]
    fn updated_renderspeed() {
        let mut player_state = PlayerState::default();

        player_state.stats.game_stats.width = 1000;
        player_state.stats.game_stats.height = 1000;

        let mut rogue_game = Rogue::new(&Rc::new(RefCell::new(player_state)));

        let start_time = Instant::now();

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = flatten_to_span(&rogue_game, None);
        let _ = spans_to_text(spans);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = flatten_to_span(&rogue_game, None);
        let _ = spans_to_text(spans);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = flatten_to_span(&rogue_game, None);
        let _ = spans_to_text(spans);

        let elapsed = start_time.elapsed().as_millis();

        println!("UpdateRenderspeed Time taken: {elapsed}");

        #[cfg(not(debug_assertions))]
        assert!(elapsed < 500);
    }
}
