//! This module implements the core game logic for the roguelike.
//! It manages game state, character movement, enemy behavior, and rendering.

#[cfg(not(target_family = "wasm"))]
use std::time::{Duration, Instant};

#[cfg(target_family = "wasm")]
use web_time::{Duration, Instant};

use crate::common::{
    TICK_RATE,
    carnagereport::CarnageReport,
    center,
    character::{Character, Damageable, Movable},
    coords::{Area, Direction, Position},
    effects::DamageEffect,
    enemy::*,
    pickups::{Pickupable, PowerupOrb},
    timescaler::TimeScaler,
    upgrade::PlayerState,
};
use crate::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

pub type Layer = Vec<Vec<EntityCharacters>>;

/// Represents the main game state and logic.
pub struct RogueGame {
    /// The player's current state, including stats and inventory.
    pub player_state: PlayerState,

    /// The carnage report, which is displayed at the end of a level.
    pub carnage_report: Option<CarnageReport>,

    /// The rendered map text.
    pub map_text: Text<'static>,

    character: Character,
    layer_base: Layer,
    layer_pickups: Layer,
    layer_entities: Layer,
    layer_effects: Layer,

    tickcount: u64,

    height: usize,
    width: usize,

    enemies: Vec<Enemy>,

    enemy_spawn_ticks: u64,
    enemy_move_ticks: u64,

    enemy_health: i32,
    enemy_damage: i32,
    enemy_worth: u32,

    attack_ticks: u64,

    /// A flag indicating whether the game is over.
    pub game_over: bool,
    /// A flag indicating whether the game should exit.
    pub exit: bool,

    active_damage_effects: Vec<DamageEffect>,

    pickups: Vec<Box<dyn Pickupable>>,

    timer: Duration,
    start_time: Instant,

    timescaler: TimeScaler,

    view_area: Rect,
    camera_area: Area,
}

impl RogueGame {
    pub fn new(player_state: PlayerState) -> Self {
        let width = player_state.stats.game_stats.width;
        let height = player_state.stats.game_stats.height;

        let mut base: Layer = Vec::from(Vec::new());
        let mut entities: Layer = Vec::from(Vec::new());
        let mut effects: Layer = Vec::from(Vec::new());
        let mut pickups: Layer = Vec::from(Vec::new());

        let mut rng = rand::rng();

        for _ in 0..height {
            let mut baseline = Vec::new();
            let mut entityline = Vec::new();
            let mut effectsline = Vec::new();
            let mut pickupsline = Vec::new();
            for _ in 0..width {
                let choice = rng.random_range(0..=1);
                match choice {
                    0 => baseline.push(EntityCharacters::Background1),
                    _ => baseline.push(EntityCharacters::Background2),
                }
                entityline.push(EntityCharacters::Empty);
                effectsline.push(EntityCharacters::Empty);
                pickupsline.push(EntityCharacters::Empty);
            }
            base.push(baseline);
            entities.push(entityline);
            effects.push(effectsline);
            pickups.push(pickupsline);
        }

        let attack_ticks = Self::per_sec_to_tick_count(1.5);
        let enemy_move_ticks = Self::per_sec_to_tick_count(2.);
        let enemy_spawn_ticks =
            Self::per_sec_to_tick_count(0.4 * player_state.stats.game_stats.enemy_spawn_mult);

        let start_time = Instant::now();
        let timer = Duration::from_secs(player_state.stats.game_stats.timer);

        let mut game = RogueGame {
            player_state: player_state.clone(),
            character: Character::new(player_state.clone()),
            layer_base: base,
            layer_entities: entities,
            layer_effects: effects,
            layer_pickups: pickups,
            height,
            width,
            attack_ticks,
            enemy_move_ticks,
            enemy_spawn_ticks,

            map_text: Text::from(""),

            carnage_report: None,
            exit: false,

            enemy_damage: 1,
            enemy_health: 3,
            enemy_worth: 1,

            tickcount: 0,
            enemies: vec![],
            pickups: vec![],
            game_over: false,
            active_damage_effects: vec![],
            start_time,
            timer,
            timescaler: TimeScaler::now()
                .offset_start_time(player_state.stats.game_stats.time_offset),

            view_area: Rect::new(0, 0, width as u16, height as u16),
            camera_area: Area::new(Position(0, 0), Position(width as i32, height as i32)),
        };

        game.init_character();
        game.update_stats();

        if game.player_state.upgrade_owned("53") {
            game.spawn_orb();
        }

        game
    }

    pub fn per_sec_to_tick_count(per_sec: f64) -> u64 {
        let per_tick = TICK_RATE / per_sec;
        per_tick.ceil() as u64
    }

    pub fn spawn_orb(&mut self) {
        let position = get_rand_position_on_layer(&self.layer_base);

        self.pickups.push(Box::new(PowerupOrb::new(position)));
    }

    pub fn on_tick(&mut self) {
        if self.game_over {
            return;
        }

        self.tickcount += 1;

        if self.start_time.elapsed() >= self.timer {
            self.game_over = true;
        }

        if !self.character.is_alive() {
            self.game_over = true;
        }

        let mut debuffed_enemies: Vec<Enemy> = Vec::new();

        self.enemies = self
            .enemies
            .clone()
            .into_iter()
            .filter(|e| {
                if !e.is_alive() {
                    if e.debuffs.len() > 0 {
                        debuffed_enemies.push(e.clone());
                    }
                    self.player_state.inventory.add_gold(e.get_worth());

                    return false;
                } else {
                    return true;
                };
            })
            .collect();

        debuffed_enemies.into_iter().for_each(|e| {
            e.debuffs
                .iter()
                .map(|d| d.on_death(e.clone()))
                .for_each(|maybe_damage_area| {
                    if let Some(mut damage_area) = maybe_damage_area {
                        damage_area.area.constrain(&self.layer_entities.clone());
                        damage_area.deal_damage(&mut self.enemies);

                        let damage_effect = DamageEffect::from(damage_area);

                        self.active_damage_effects.push(damage_effect);
                    }
                });
        });

        if self.tickcount % self.enemy_spawn_ticks == 0 {
            self.spawn_enemy();
        }

        if self.tickcount % self.enemy_move_ticks == 0 {
            self.enemies.iter_mut().for_each(|enemy| {
                enemy.update(
                    &mut self.character,
                    &self.layer_entities,
                    &mut self.active_damage_effects,
                );
                // update_entity_positions(&mut self.layer_entities, enemy);

                if self.character.stats.shove_amount > 0
                    && is_next_to_character(self.character.get_pos(), enemy.get_prev_pos())
                {
                    if self.character.stats.shove_damage > 0 {
                        enemy.take_damage(
                            (self.character.stats.shove_damage as f64
                                * self.character.stats.damage_mult)
                                .ceil() as i32,
                        );
                    }

                    enemy.move_back(
                        self.character.stats.shove_amount as i32,
                        &self.layer_entities,
                    );
                }
            });
            // self.change_low_health_enemies_questionable();
        }

        if self.tickcount % TICK_RATE.floor() as u64 == 0 {
            self.scale();
            self.scale_enemies();
        }

        if self.tickcount % self.attack_ticks == 0 {
            let (damage_areas, mut damage_effects) = self.character.attack(&mut self.layer_effects);
            damage_areas.iter().for_each(|area| {
                area.deal_damage(&mut self.enemies);
            });
            self.active_damage_effects.append(&mut damage_effects)
        }

        update_layer_entities(&mut self.layer_entities, &self.enemies, &self.character);

        self.pickups
            .iter_mut()
            .for_each(|pickup| pickup.animate(self.tickcount % 1000));
        update_layer_pickups(&mut self.layer_pickups, &self.pickups);

        self.camera_area =
            get_camera_area(self.view_area, self.get_character_pos(), &self.layer_base);

        let spans = self.flatten_to_span(Some(self.camera_area.clone()));

        self.map_text = Self::spans_to_text(spans);
    }

    pub fn on_frame(&mut self) {
        self.active_damage_effects = self
            .active_damage_effects
            .clone()
            .into_iter()
            .map(|mut damage_effect| {
                damage_effect.update(&mut self.layer_effects);
                damage_effect
            })
            .filter(|damage_effect| !damage_effect.complete)
            .collect();

        // self.change_low_health_enemies_questionable();
    }

    pub fn update_stats(&mut self) {
        self.attack_ticks = (self.attack_ticks as f64
            / self.player_state.stats.game_stats.attack_speed_mult)
            .ceil() as u64;
    }

    fn scale_enemies(&mut self) {
        let init_enemy_health = 3.;
        let init_enemy_damage = 1.;
        let init_enemy_spawn_secs = 0.4 * self.player_state.stats.game_stats.enemy_spawn_mult;
        let init_enemy_move_secs = 2. * self.player_state.stats.game_stats.enemy_move_mult;
        let init_enemy_worth: u32 = 1;

        self.enemy_health =
            (init_enemy_health * (self.timescaler.scale_amount).max(1.)).ceil() as i32;

        self.enemy_damage =
            (init_enemy_damage * (self.timescaler.scale_amount / 5.).max(1.)).ceil() as i32;
        self.enemy_spawn_ticks = Self::per_sec_to_tick_count(
            init_enemy_spawn_secs as f64 * self.timescaler.scale_amount,
        );

        self.enemy_move_ticks = Self::per_sec_to_tick_count(
            init_enemy_move_secs as f64 * (self.timescaler.scale_amount / 3.5).max(1.),
        );

        self.enemy_worth =
            (init_enemy_worth as f64 * (self.timescaler.scale_amount / 2.).max(1.)).ceil() as u32;
    }

    pub fn spawn_enemy(&mut self) {
        self.enemies.push(Enemy::new(
            get_rand_position_on_edge(&self.layer_entities),
            self.enemy_damage,
            self.enemy_health,
            self.enemy_worth,
        ))
    }

    fn scale(&mut self) -> f64 {
        self.timescaler.scale()
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Some(_) = self.carnage_report {
            match key_event.code {
                KeyCode::Esc => self.exit = true,
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Down => move_entity(
                    &mut self.layer_entities,
                    &mut self.character,
                    Direction::DOWN,
                ),
                KeyCode::Char('w') | KeyCode::Up => {
                    move_entity(&mut self.layer_entities, &mut self.character, Direction::UP)
                }
                KeyCode::Char('d') | KeyCode::Right => move_entity(
                    &mut self.layer_entities,
                    &mut self.character,
                    Direction::RIGHT,
                ),
                KeyCode::Char('a') | KeyCode::Left => move_entity(
                    &mut self.layer_entities,
                    &mut self.character,
                    Direction::LEFT,
                ),
                KeyCode::Esc => self.game_over = true,
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

    pub fn flatten_to_span(&self, area: Option<Area>) -> Vec<Vec<Span<'static>>> {
        let (mut x1, mut y1, mut x2, mut y2) = Area::from(self.layer_base.clone()).get_bounds();

        if let Some(inner_area) = area {
            (x1, y1, x2, y2) = inner_area.get_bounds();
        }

        let out: Vec<(usize, Vec<(usize, Span<'static>)>)> = self
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

        let out: Vec<Vec<Span<'_>>> = out
            .into_iter()
            .map(|(y, row)| {
                row.into_iter()
                    .map(|(x, mut pos)| {
                        if self.layer_effects[y][x] != EntityCharacters::Empty {
                            pos.clone_from(&self.layer_effects[y][x].to_styled());
                        } else if self.layer_entities[y][x] != EntityCharacters::Empty {
                            pos.clone_from(&self.layer_entities[y][x].to_styled());
                        } else if self.layer_pickups[y][x] != EntityCharacters::Empty {
                            pos.clone_from(&self.layer_pickups[y][x].to_styled());
                        }
                        pos
                    })
                    .collect()
            })
            .collect();

        out
    }

    pub fn spans_to_text(spans: Vec<Vec<Span<'_>>>) -> Text<'_> {
        let map = spans;

        let out: Text<'_> = map
            .into_iter()
            .map(|style_line| Line::default().spans(style_line))
            .collect();

        out
    }

    /// Returns the character's current position.
    pub fn get_character_pos(&self) -> &Position {
        self.character.get_pos()
    }

    pub fn can_stand(&self, position: &Position) -> bool {
        let (x, y) = position.get();
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return false;
        }
        get_pos(&self.layer_entities, position) == &EntityCharacters::Empty
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let timer = self.timer.saturating_sub(self.start_time.elapsed());

        let title = Line::from(" dispair.run ".bold());

        let instructions = Line::from(vec![
            " Health: ".dark_gray(),
            self.character.get_health().to_string().bold(),
            " ".into(),
            " Time: ".dark_gray(),
            timer.as_secs().to_string().bold().into(),
            " ".into(),
            " Gold: ".dark_gray(),
            self.player_state.inventory.gold.to_string().into(),
            " ".into(),
        ]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.right_aligned())
            .border_set(border::THICK);

        self.view_area = block.inner(frame.area());

        let content_area = self.view_area;

        let height = self.map_text.lines.len() as u16;
        let width = self.map_text.lines[0].iter().len() as u16;

        let centered_area = center(content_area, width, height);

        let content = Paragraph::new(self.map_text.clone()).centered();

        frame.render_widget(block, frame.area());
        frame.render_widget(content, centered_area);

        if let Some(ref mut carnage) = self.carnage_report.clone() {
            carnage.render(frame);
        }
    }
}

pub fn get_camera_area(content_area: Rect, player_pos: &Position, layer: &Layer) -> Area {
    let view_height = content_area.height as i32;
    let view_width = content_area.width as i32;

    let layer_height = layer.len() as i32;
    let layer_width = layer[0].len() as i32;

    let (player_x, player_y) = player_pos.get();

    // Center the camera on the player
    let mut camera_x1 = player_x - view_width / 2;
    let mut camera_y1 = player_y - view_height / 2;
    let mut camera_x2 = camera_x1 + view_width;
    let mut camera_y2 = camera_y1 + view_height;

    // Clamp to left edge
    if camera_x1 < 0 {
        camera_x1 = 0;
        camera_x2 = view_width;
    }

    // Clamp to top edge
    if camera_y1 < 0 {
        camera_y1 = 0;
        camera_y2 = view_height;
    }

    // Clamp to right edge
    if camera_x2 > layer_width {
        camera_x2 = layer_width;
        camera_x1 = (layer_width - view_width).max(0);
    }

    // Clamp to bottom edge
    if camera_y2 > layer_height {
        camera_y2 = layer_height;
        camera_y1 = (layer_height - view_height).max(0);
    }

    Area {
        corner1: Position(camera_x1, camera_y1),
        corner2: Position(camera_x2, camera_y2),
    }
}

pub fn get_pos<'a>(layer: &'a Layer, position: &Position) -> &'a EntityCharacters {
    let (x, y) = position.get_as_usize();
    &layer[y][x]
}

pub fn clear_layer(layer: &mut Layer) {
    layer.iter_mut().for_each(|row| {
        row.iter_mut()
            .for_each(|ent| ent.replace(EntityCharacters::Empty))
    });
}

pub fn update_layer_entities(
    layer_entities: &mut Layer,
    enemies: &Vec<Enemy>,
    character: &Character,
) {
    clear_layer(layer_entities);

    enemies.iter().for_each(|enemy| {
        let (x, y) = enemy.get_pos().get_as_usize();

        layer_entities[y][x] = enemy.get_entity_char();
    });

    let (char_x, char_y) = character.get_pos().get_as_usize();
    layer_entities[char_y][char_x] = character.get_entity_char();
}

pub fn update_layer_pickups(layer_pickups: &mut Layer, pickups: &Vec<Box<dyn Pickupable>>) {
    clear_layer(layer_pickups);

    pickups.iter().for_each(|pickup| {
        let (x, y) = pickup.get_pos().get_as_usize();

        layer_pickups[y][x] = pickup.get_entity_char().clone();
    });
}

pub fn set_entity(
    layer: &mut Vec<Vec<EntityCharacters>>,
    position: &Position,
    entity: EntityCharacters,
) -> Result<(), String> {
    let mut position = position.clone();
    position.constrain(layer);
    let (x, y) = position.get_as_usize();
    if x >= layer[0].len() || y >= layer.len() {
        return Err("Position out of bounds".to_string());
    }
    layer[y][x] = entity;
    Ok(())
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

pub fn can_stand(layer: &Layer, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    x < layer[0].len() && y < layer.len() && layer[y][x] == EntityCharacters::Empty
}

pub fn get_rand_position_on_edge(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let which_edge = rng.random_range(0..4);
    let position = match which_edge {
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
    };
    position
}

pub fn get_rand_position_on_layer(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let x = rng.random_range(0..layer[0].len() as i32);
    let y = rng.random_range(0..layer.len() as i32);
    Position::new(x, y)
}

pub fn is_next_to_character(char_position: &Position, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    let (char_x, char_y) = char_position.get_as_usize();

    if x == char_x.saturating_add(1) && y == char_y
        || x == char_x.saturating_sub(1) && y == char_y
        || y == char_y.saturating_add(1) && x == char_x
        || y == char_y.saturating_sub(1) && x == char_x
    {
        true
    } else {
        false
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum EntityCharacters {
    Background1,
    Background2,
    Character(Style),
    Enemy(Style),
    Empty,
    AttackBlackout(Style),
    Orb(Style),
}

impl EntityCharacters {
    pub fn to_styled(&self) -> Span<'static> {
        match self {
            EntityCharacters::Background1 => Span::from(".").dark_gray(),
            EntityCharacters::Background2 => Span::from(",").dark_gray(),
            EntityCharacters::Character(style) => {
                Span::from("0").white().bold().style(style.clone())
            }
            EntityCharacters::Enemy(style) => Span::from("x").white().style(style.clone()),
            EntityCharacters::Empty => Span::from(" "),
            EntityCharacters::AttackBlackout(style) => {
                Span::from(ratatui::symbols::block::FULL).style(style.clone())
            }
            EntityCharacters::Orb(style) => Span::from("o").style(style.clone()),
        }
    }

    pub fn replace(&mut self, new_entity: EntityCharacters) {
        *self = new_entity;
    }

    pub fn style_mut(&mut self) -> &mut Style {
        match self {
            EntityCharacters::Character(style) => style,
            EntityCharacters::Enemy(style) => style,
            _ => panic!("Cannot get style_mut for non-styled entity"),
        }
    }

    /// Checks if the entity is a player character.
    pub fn is_char(&self) -> bool {
        match self {
            EntityCharacters::Character(_) => true,
            _ => false,
        }
    }
}
