//! This module implements the core game logic for the roguelike.
//! It manages game state, character movement, enemy behavior, and rendering.

use crate::common::character::Renderable;
use crate::common::enemies::enemy::{Enemy, EnemyDrops};
use crate::common::enemies::enemywrangler::EnemyWrangler;
use crate::common::pickups::PickupTypes;
use crate::common::pickups::pickupwrangler::PickupWrangler;
use crate::common::{Goto, Viewable};
use crate::{
    common::{
        TICK_RATE, center,
        character::{Character, Damageable, Movable},
        coords::{Area, Direction, Position, SquareArea},
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
    text::{Line, Span, Text},
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
    pub player_state: Rc<RefCell<PlayerState>>,
    init_state: PlayerState,

    /// The carnage report, which is displayed at the end of a level.
    pub carnage_report: Option<CarnageReport>,

    pub powerup_popup: Option<PowerupPopup>,

    /// The rendered map text.
    pub map_text: Text<'static>,

    character: Character,
    layer_base: Layer,
    pub flat_layer: Layer,

    tickcount: u64,

    height: usize,
    width: usize,

    enemies: Rc<RefCell<Vec<Enemy>>>,

    enemy_wrangler: EnemyWrangler,

    attack_ticks: u64,

    pub game_state: GameState,

    pub goto: Goto,

    active_damage_effects: Vec<DamageEffect>,

    pickup_wrangler: PickupWrangler,

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
    pub fn new(player_state: &Rc<RefCell<PlayerState>>) -> Self {
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

        let attack_ticks = Self::per_sec_to_tick_count(Self::DEFAULT_ATTACK_P_S);

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

    #[must_use]
    pub fn per_sec_to_tick_count(per_sec: f64) -> u64 {
        let per_tick = TICK_RATE / per_sec;
        per_tick.ceil() as u64
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

                self.player_state
                    .borrow_mut()
                    .upgrades
                    .insert("A".to_string(), 1);
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

            let spans = self.flatten_to_span(Some(self.camera_area.clone()));

            self.map_text = Self::spans_to_text(spans);
        }
    }

    pub fn update_stats(&mut self) {
        self.attack_ticks = Self::per_sec_to_tick_count(Self::DEFAULT_ATTACK_P_S);
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

    #[must_use]
    pub fn flatten_to_span(&self, area: Option<SquareArea>) -> Vec<Vec<Span<'static>>> {
        fn callback_creator<F: std::borrow::Borrow<T>, T: Renderable>(
            enum_2d: &mut Vec<(usize, Vec<(usize, Span)>)>,
            layer: &Layer,
        ) -> impl FnMut(F) {
            |entity: F| {
                let mut pos = entity.borrow().get_pos().clone();
                pos.constrain(layer);

                if let Some(entity_pos) = Rogue::get_mut_item_in_2d_enum_vec(enum_2d, &pos) {
                    *entity_pos = entity.borrow().get_entity_char().to_styled();
                }
            }
        }

        let (x1, y1, x2, y2);
        if let Some(inner_area) = area {
            (x1, y1, x2, y2) = inner_area.get_bounds();
        } else {
            (x1, y1, x2, y2) = (
                0,
                0,
                self.layer_base[0].len() as i32 - 1,
                self.layer_base.len() as i32 - 1,
            );
        }

        let mut enum_2d: Vec<(usize, Vec<(usize, Span<'static>)>)> = self
            .layer_base
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if i >= y1 as usize && i <= y2 as usize {
                    Some((
                        i,
                        line.iter()
                            .enumerate()
                            .filter_map(|(i, entity)| {
                                if i >= x1 as usize && i <= x2 as usize {
                                    Some((i, entity.to_styled()))
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        self.pickup_wrangler
            .pickups
            .iter()
            .for_each(callback_creator::<_, PickupTypes>(
                &mut enum_2d,
                &self.layer_base,
            ));

        self.enemies
            .borrow()
            .iter()
            .for_each(callback_creator::<_, Enemy>(&mut enum_2d, &self.layer_base));

        self.active_damage_effects.iter().for_each(|effect| {
            effect
                .get_instructions()
                .for_each(callback_creator(&mut enum_2d, &self.layer_base));
        });

        {
            let mut character_callback =
                callback_creator::<_, Character>(&mut enum_2d, &self.layer_base);
            character_callback(&self.character);
        }

        enum_2d
            .into_iter()
            .map(|(_, vec)| vec.into_iter().map(|(_, item)| item).collect())
            .collect()
    }

    pub fn get_mut_item_in_2d_enum_vec<'a, T>(
        vec: &'a mut [(usize, Vec<(usize, T)>)],
        position: &'a Position,
    ) -> Option<&'a mut T> {
        let (x, y) = position.get_as_usize();
        let maybe_row = vec.iter_mut().find(|(in_y, _)| in_y == &y);
        if let Some(row) = maybe_row {
            let maybe_item = row.1.iter_mut().find(|(in_x, _)| in_x == &x);
            if let Some(item) = maybe_item {
                Some(&mut item.1)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_enemy_positions(&self) -> Vec<Position> {
        self.enemies
            .borrow()
            .iter()
            .map(|enemy| enemy.get_pos().clone())
            .collect()
    }

    #[must_use]
    pub fn spans_to_text(spans: Vec<Vec<Span<'_>>>) -> Text<'_> {
        let map = spans;

        let out: Text<'_> = map
            .into_iter()
            .map(|style_line| Line::default().spans(style_line))
            .collect();

        out
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

    // #[must_use]
    // pub fn can_stand(&self, position: &Position) -> bool {
    //     let (x, y) = position.get();
    //
    //     if x < 0
    //         || x >= self.width as i32
    //         || y < 0
    //         || y >= self.height as i32
    //         || position == self.get_character_pos()
    //     // || self.get_enemy_positions().contains(position)
    //     {
    //         return false;
    //     }
    //     true
    // }

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
#[must_use]
pub fn get_camera_area(content_area: Rect, player_pos: &Position, layer: &Layer) -> SquareArea {
    let view_height = i32::from(content_area.height);
    let view_width = i32::from(content_area.width);

    let layer_height = layer.len() as i32;
    let layer_width = layer[0].len() as i32;

    let (player_x, player_y) = player_pos.get();

    // Center the camera on the player
    let mut x1 = player_x - view_width / 2;
    let mut y1 = player_y - view_height / 2;
    let mut x2 = x1 + view_width;
    let mut y2 = y1 + view_height;

    // Clamp to the left edge
    if x1 < 0 {
        x1 = 0;
        x2 = view_width;
    }

    // Clamp to top edge
    if y1 < 0 {
        y1 = 0;
        y2 = view_height;
    }

    // Clamp to right edge
    if x2 > layer_width {
        x2 = layer_width;
        x1 = (layer_width - view_width).max(0);
    }

    // Clamp to bottom edge
    if y2 > layer_height {
        y2 = layer_height;
        y1 = (layer_height - view_height).max(0);
    }

    SquareArea {
        corner1: Position(x1, y1),
        corner2: Position(x2, y2),
    }
}

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

#[must_use]
pub fn get_pos<'a>(layer: &'a Layer, position: &Position) -> &'a EntityCharacters {
    let (x, y) = position.get_as_usize();
    &layer[y][x]
}

pub fn update_effects(damage_effects: &mut [DamageEffect]) {
    for effect in damage_effects.iter_mut() {
        effect.update();
    }
}

pub fn move_entity(layer: &mut Layer, entity: &mut impl Movable, direction: Direction) {
    let (x, y) = entity.get_pos().get();
    let mut new_pos = match direction {
        Direction::LEFT => Position::new(x - 1, y),
        Direction::RIGHT => Position::new(x + 1, y),
        Direction::UP => Position::new(x, y - 1),
        Direction::DOWN => Position::new(x, y + 1),
    };

    new_pos.constrain(layer);

    if can_stand(layer, &new_pos) {
        entity.move_to(new_pos, direction);
        // update_entity_positions(layer, entity);
    } else {
        entity.move_to(entity.get_pos().clone(), direction);
    }
}

#[must_use]
pub fn can_stand(layer: &Layer, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    x < layer[0].len() && y < layer.len()
}

#[must_use]
pub fn get_rand_position_on_edge(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let which_edge = rng.random_range(0..4);

    match which_edge {
        0 => Position::new(0, rng.random_range(0..layer.len() as i32)),
        1 => Position::new(
            layer[0].len() as i32 - 1,
            rng.random_range(0..layer.len() as i32),
        ),
        2 => Position::new(rng.random_range(0..layer[0].len() as i32), 0),
        3 => Position::new(
            rng.random_range(0..layer[0].len() as i32),
            layer.len() as i32 - 1,
        ),
        _ => Position::new(0, 0),
    }
}

#[must_use]
pub fn get_rand_position_on_layer(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let x = rng.random_range(0..layer[0].len() as i32);
    let y = rng.random_range(0..layer.len() as i32);
    Position::new(x, y)
}

#[must_use]
pub fn is_next_to_character(char_position: &Position, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    let (char_x, char_y) = char_position.get_as_usize();

    (x == char_x.saturating_add(1) || x == char_x.saturating_sub(1)) && y == char_y
        || (y == char_y.saturating_add(1) || y == char_y.saturating_sub(1)) && x == char_x
}

#[derive(PartialEq, Eq, Clone)]
pub enum EntityCharacters {
    Background1,
    Background2,
    Character(Style),
    Enemy(Style),
    Empty,
    AttackBlackout(Style),
    AttackMist(Style),
    AttackWeak(Style),
    Orb(Style),
}

impl EntityCharacters {
    #[must_use]
    pub fn to_styled(&self) -> Span<'static> {
        match self {
            EntityCharacters::Background1 => Span::from(".").dark_gray(),
            EntityCharacters::Background2 => Span::from(",").dark_gray(),
            EntityCharacters::Character(style) => Span::from("0").white().bold().style(*style),
            EntityCharacters::Enemy(style) => Span::from("x").white().style(*style),
            EntityCharacters::Empty => Span::from(" "),
            EntityCharacters::AttackBlackout(style) => {
                Span::from(ratatui::symbols::block::FULL).style(*style)
            }
            EntityCharacters::AttackMist(style) => {
                Span::from(ratatui::symbols::shade::MEDIUM).style(*style)
            }
            EntityCharacters::AttackWeak(style) => {
                Span::from(ratatui::symbols::shade::LIGHT).style(*style)
            }
            EntityCharacters::Orb(style) => Span::from("o").style(*style),
        }
    }

    pub fn replace(&mut self, new_entity: EntityCharacters) {
        *self = new_entity;
    }

    /// Get mutable reference to inner style if it exists.
    ///
    /// # Panics
    ///
    /// If trying to call `style_mut` on an `EntityCharacters` which does not have an inner style, it will panic.
    pub fn style_mut(&mut self) -> &mut Style {
        match self {
            EntityCharacters::Character(style)
            | EntityCharacters::Enemy(style)
            | EntityCharacters::Orb(style)
            | EntityCharacters::AttackBlackout(style)
            | EntityCharacters::AttackMist(style)
            | EntityCharacters::AttackWeak(style) => style,
            _ => panic!("Cannot get style_mut for a non-styled entity"),
        }
    }

    /// Checks if the entity is a player character.
    #[must_use]
    pub fn is_char(&self) -> bool {
        matches!(self, EntityCharacters::Character(_))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Instant;

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

        let spans = rogue_game.flatten_to_span(None);
        let _ = Rogue::spans_to_text(spans);

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

        let spans = rogue_game.flatten_to_span(None);
        let _ = Rogue::spans_to_text(spans);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = rogue_game.flatten_to_span(None);
        let _ = Rogue::spans_to_text(spans);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = rogue_game.flatten_to_span(None);
        let _ = Rogue::spans_to_text(spans);

        let elapsed = start_time.elapsed().as_millis();

        println!("UpdateRenderspeed Time taken: {elapsed}");

        #[cfg(not(debug_assertions))]
        assert!(elapsed < 500);
    }
}
