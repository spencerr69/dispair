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
    /// Creates a new RogueGame initialized from the given player state.
    ///
    /// The returned game is configured with map layers, timing cadences, initial
    /// enemy stats, a player character placed on the map, and a timescaler offset
    /// from the player's game stats. If the player owns upgrade "53", an orb
    /// pickup is spawned on the map.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `player_state` is a valid PlayerState configured for a game.
    /// let game = RogueGame::new(player_state);
    /// // The game map width matches the player's configured width.
    /// assert_eq!(game.width as usize, game.player_state.stats.game_stats.width as usize);
    /// ```
    pub fn new(player_state: PlayerState) -> Self {
        let width = player_state.stats.game_stats.width;
        let height = player_state.stats.game_stats.height;

        let mut base: Layer = Vec::from(Vec::new());

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

    /// Advance the game state by one tick, progressing timers, resolving deaths, enemy and player actions, and updating rendering layers.
    ///
    /// This updates internal game timing and may set `game_over`. Dead enemies are removed and processed (rewards and death effects), enemies may spawn or move according to cadence, player attacks are resolved and damage effects are queued, pickups animate, entity/effect/pickup layers are refreshed, the camera area is recomputed, and `map_text` is regenerated for the current view.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Constructing a full `RogueGame` is out of scope for this example.
    /// // The important behavior is that calling `on_tick()` advances internal tick state.
    /// let mut game = /* create or obtain a RogueGame instance */;
    /// let before = game.tickcount;
    /// game.on_tick();
    /// assert_eq!(game.tickcount, before + 1);
    /// ```
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

        self.enemies.retain(|e| {
            if !e.is_alive() {
                if e.debuffs.len() > 0 {
                    debuffed_enemies.push(e.clone());
                }
                self.player_state.inventory.add_gold(e.get_worth());

                return false;
            } else {
                return true;
            };
        });

        debuffed_enemies.into_iter().for_each(|e| {
            e.debuffs
                .iter()
                .map(|d| d.on_death(e.clone()))
                .for_each(|maybe_damage_area| {
                    if let Some(mut damage_area) = maybe_damage_area {
                        damage_area.area.constrain(&self.layer_base);
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
                    &self.layer_base,
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

                    enemy.move_back(self.character.stats.shove_amount as i32, &self.layer_base);
                }
            });
            // self.change_low_health_enemies_questionable();
        }

        if self.tickcount % TICK_RATE.floor() as u64 == 0 {
            self.scale();
            self.scale_enemies();
        }

        if self.tickcount % self.attack_ticks == 0 {
            let (damage_areas, mut damage_effects) = self.character.attack(&mut self.layer_base);
            damage_areas.iter().for_each(|area| {
                area.deal_damage(&mut self.enemies);
            });
            self.active_damage_effects.append(&mut damage_effects)
        }

        self.pickups
            .iter_mut()
            .for_each(|pickup| pickup.animate(self.tickcount % 1000));

        self.camera_area =
            get_camera_area(self.view_area, self.get_character_pos(), &self.layer_base);
    }

    /// Advance per-frame visual effects and remove completed damage effects from the game state.
    ///
    /// This updates the effects rendering layer from `active_damage_effects` and then
    /// filters out any damage effects marked complete so they no longer persist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Given a mutable `RogueGame` instance `game`:
    /// // let mut game = /* initialize RogueGame */;
    /// game.on_frame();
    /// ```
    pub fn on_frame(&mut self) {
        update_effects(&mut self.active_damage_effects);
        self.active_damage_effects.retain(|effect| !effect.complete);

        let spans = self.flatten_to_span(Some(self.camera_area.clone()));

        self.map_text = Self::spans_to_text(spans);

        // self.change_low_health_enemies_questionable();
    }

    /// Adjust the attack tick cadence using the player's attack speed multiplier.
    ///
    /// This updates `self.attack_ticks` by dividing it by `player_state.stats.game_stats.attack_speed_mult`
    /// and rounding the result up to the next integer, reducing the number of ticks between attacks
    /// as the multiplier increases.
    ///
    /// # Examples
    ///
    /// ```
    /// // Equivalent local calculation shown for clarity:
    /// let mut attack_ticks = 10u64;
    /// let attack_speed_mult = 1.5f64;
    /// attack_ticks = (attack_ticks as f64 / attack_speed_mult).ceil() as u64;
    /// assert_eq!(attack_ticks, 7);
    /// ```
    pub fn update_stats(&mut self) {
        self.attack_ticks = (self.attack_ticks as f64
            / self.player_state.stats.game_stats.attack_speed_mult)
            .ceil() as u64;
    }

    /// Update enemy attributes and spawn/movement cadences according to the current
    /// timescaler and the player's game stats.
    ///
    /// This adjusts the following fields on self:
    /// - `enemy_health`
    /// - `enemy_damage`
    /// - `enemy_spawn_ticks`
    /// - `enemy_move_ticks`
    /// - `enemy_worth`
    ///
    /// The values are scaled from internal base constants using `timescaler.scale_amount`
    /// and multipliers stored in `player_state.stats.game_stats`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a mutable `game: RogueGame` with `timescaler` and `player_state` initialized,
    /// // call `scale_enemies()` to recompute enemy stats for the current difficulty.
    /// //
    /// // game.scale_enemies();
    /// ```
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

    /// Spawns a new enemy at a random position on the edge of the map.
    pub fn spawn_enemy(&mut self) {
        self.enemies.push(Enemy::new(
            get_rand_position_on_edge(&self.layer_base),
            self.enemy_damage,
            self.enemy_health,
            self.enemy_worth,
        ))
    }

    /// Updates the time scaler and returns the current scale amount.
    fn scale(&mut self) -> f64 {
        self.timescaler.scale()
    }

    /// Handles key events for the game, such as movement and pausing.
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Some(_) = self.carnage_report {
            match key_event.code {
                KeyCode::Esc => self.exit = true,
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Down => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::DOWN)
                }
                KeyCode::Char('w') | KeyCode::Up => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::UP)
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::RIGHT)
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    move_entity(&mut self.layer_base, &mut self.character, Direction::LEFT)
                }
                KeyCode::Esc => self.game_over = true,
                _ => {}
            }
        }
    }

    /// Place the player character at a random position within the map bounds.
    ///
    /// The character's position is set to an (x, y) coordinate where
    /// 0 <= x < width and 0 <= y < height.
    ///
    /// # Examples
    ///
    /// ```
    /// // assuming `game` is a mutable `RogueGame` instance
    /// game.init_character();
    /// let pos = game.get_character_pos();
    /// assert!(pos.0 >= 0 && pos.0 < game.width as i32);
    /// assert!(pos.1 >= 0 && pos.1 < game.height as i32);
    /// ```
    pub fn init_character(&mut self) {
        let mut rng = rand::rng();

        let (x, y) = (
            rng.random_range(0..self.width) as i32,
            rng.random_range(0..self.height) as i32,
        );

        self.character.set_pos(Position(x, y));
    }

    /// Produce a 2D grid of styled spans for rendering the visible cells within a given area.
    ///
    /// When `area` is `None`, the entire base layer is used. The returned vector is ordered by
    /// rows (top to bottom), and each inner vector contains the styled `Span<'static>` values for
    /// the visible columns in that row. Layer precedence is applied: effects override entities,
    /// entities override pickups, and pickups override the base layer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // `game` is a `RogueGame` instance.
    /// let rows = game.flatten_to_span(None);
    /// assert!(rows.len() > 0);
    /// ```
    pub fn flatten_to_span(&self, area: Option<Area>) -> Vec<Vec<Span<'static>>> {
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

        self.pickups.iter().for_each(|pickup| {
            if let Some(pickup_pos) =
                Self::get_mut_item_in_2d_enum_vec(&mut enum_2d, pickup.get_pos())
            {
                *pickup_pos = pickup.get_entity_char().to_styled();
            }
        });

        self.enemies.iter().for_each(|enemy| {
            if let Some(enemy_place) =
                Self::get_mut_item_in_2d_enum_vec(&mut enum_2d, enemy.get_pos())
            {
                *enemy_place = enemy.get_entity_char().to_styled();
            }
        });

        self.active_damage_effects.iter().for_each(|effect| {
            effect.get_instructions().for_each(|(mut pos, entity)| {
                pos.constrain(&self.layer_base);
                if let Some(effect_pos) = Self::get_mut_item_in_2d_enum_vec(&mut enum_2d, &pos) {
                    *effect_pos = entity.to_styled();
                }
            });
        });

        if let Some(character_place) =
            Self::get_mut_item_in_2d_enum_vec(&mut enum_2d, self.character.get_pos())
        {
            *character_place = self.character.get_entity_char().to_styled();
        }

        let out = enum_2d
            .into_iter()
            .map(|(_, vec)| vec.into_iter().map(|(_, item)| item).collect())
            .collect();

        out
    }

    pub fn get_mut_item_in_2d_enum_vec<'a, T>(
        vec: &'a mut Vec<(usize, Vec<(usize, T)>)>,
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
        true
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

/// Calculates the camera's visible area based on the player's position and the layer dimensions.
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

/// Gets the entity character at a specific position on the layer.
pub fn get_pos<'a>(layer: &'a Layer, position: &Position) -> &'a EntityCharacters {
    let (x, y) = position.get_as_usize();
    &layer[y][x]
}

/// Clears every cell in the provided layer by replacing each entry with `EntityCharacters::Empty`.
///
/// This mutates the layer in place.
///
/// # Examples
///
/// ```
/// let mut layer = vec![vec![EntityCharacters::Background1; 3]; 2];
/// clear_layer(&mut layer);
/// assert!(layer.iter().all(|row| row.iter().all(|ent| matches!(ent, EntityCharacters::Empty))));
/// ```
pub fn clear_layer(layer: &mut Layer) {
    layer.iter_mut().for_each(|row| {
        row.iter_mut()
            .for_each(|ent| ent.replace(EntityCharacters::Empty))
    });
}

/// Renders active damage effects into the effects layer and advances each effect's state.
///
/// This clears the provided `layer_effects`, writes every effect's current rendering instructions
/// into the layer (clamping positions to the layer bounds), and then updates each `DamageEffect`
/// so it progresses to its next frame/state.
///
/// # Examples
///
/// ```
/// use crate::common::{EntityCharacters, DamageEffect, update_layer_effects};
///
/// let width = 10;
/// let height = 5;
/// let mut layer: Vec<Vec<EntityCharacters>> = vec![vec![EntityCharacters::Empty; width]; height];
/// let mut effects: Vec<DamageEffect> = Vec::new(); // populate with effects in real use
///
/// update_layer_effects(&mut layer, &mut effects);
///
/// // With no effects the layer remains filled with `Empty`.
/// assert!(layer.iter().all(|row| row.iter().all(|c| matches!(c, EntityCharacters::Empty))));
/// ```
pub fn update_effects(damage_effects: &mut Vec<DamageEffect>) {
    damage_effects.into_iter().for_each(|effect| {
        effect.update();
    });
}

/// Draws all enemies and the player onto the entities layer, clearing previous contents first.
///
/// The function clears `layer_entities`, writes each enemy's rendered entity character at its
/// current position, then writes the player's entity character (overwriting any enemy at the same
/// cell).
///
/// # Panics
///
/// Panics if any entity position is outside the bounds of `layer_entities`.
///
/// # Examples
///
/// ```no_run
/// // Assuming `layer`, `enemies`, and `player` are initialized appropriately:
/// update_layer_entities(&mut layer, &enemies, &player);
/// ```
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

/// Updates the pickups layer by clearing it and placing all active pickups.
pub fn update_layer_pickups(layer_pickups: &mut Layer, pickups: &Vec<Box<dyn Pickupable>>) {
    clear_layer(layer_pickups);

    pickups.iter().for_each(|pickup| {
        let (x, y) = pickup.get_pos().get_as_usize();

        layer_pickups[y][x] = pickup.get_entity_char().clone();
    });
}

/// Sets a specific entity at a given position on the layer, checking for bounds.
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

/// Moves an entity in a specified direction if the target position is valid (standable).
pub fn move_entity(layer: &Layer, entity: &mut impl Movable, direction: Direction) {
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

/// Checks if a position is within the layer's bounds and is valid to stand on.
pub fn can_stand(layer: &Layer, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    x < layer[0].len() && y < layer.len()
}

/// Returns a random position on one of the four edges of the layer.
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

/// Returns a random position anywhere within the layer's bounds.
pub fn get_rand_position_on_layer(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let x = rng.random_range(0..layer[0].len() as i32);
    let y = rng.random_range(0..layer.len() as i32);
    Position::new(x, y)
}

/// Checks if a position is adjacent (up, down, left, or right) to the character's position.
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
    AttackMist(Style),
    AttackWeak(Style),
    Orb(Style),
}

impl EntityCharacters {
    /// Convert an EntityCharacters variant into a styled `Span` for rendering.
    ///
    /// Produces the textual symbol and associated `Style` used to draw this entity on the map.
    ///
    /// # Examples
    ///
    /// ```
    /// // Ensure the method compiles and returns a Span for a simple variant.
    /// let _ = crate::common::roguegame::EntityCharacters::Empty.to_styled();
    /// ```
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
            EntityCharacters::AttackMist(style) => {
                Span::from(ratatui::symbols::shade::MEDIUM).style(style.clone())
            }
            EntityCharacters::AttackWeak(style) => {
                Span::from(ratatui::symbols::shade::LIGHT).style(style.clone())
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

    /// Determine whether this entity represents the player character.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::common::roguegame::EntityCharacters;
    /// use ratatui::style::Style;
    ///
    /// let player = EntityCharacters::Character(Style::default());
    /// let enemy = EntityCharacters::Enemy(Style::default());
    ///
    /// assert!(player.is_char());
    /// assert!(!enemy.is_char());
    /// ```
    pub fn is_char(&self) -> bool {
        match self {
            EntityCharacters::Character(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::common::{roguegame::RogueGame, upgrade::PlayerState};

    #[test]
    fn renderspeed() {
        let mut player_state = PlayerState::default();

        player_state.stats.game_stats.width = 1000;
        player_state.stats.game_stats.height = 1000;

        let mut rogue_game = RogueGame::new(player_state);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let start_time = Instant::now();

        let spans = rogue_game.flatten_to_span(None);
        let _ = RogueGame::spans_to_text(spans);

        let elapsed = start_time.elapsed().as_millis();

        println!("Renderspeed Time taken: {:?}", elapsed);

        #[cfg(not(debug_assertions))]
        assert!(elapsed < 100);
    }

    #[test]
    fn updatedrenderspeed() {
        let mut player_state = PlayerState::default();

        player_state.stats.game_stats.width = 1000;
        player_state.stats.game_stats.height = 1000;

        let mut rogue_game = RogueGame::new(player_state);

        let start_time = Instant::now();

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = rogue_game.flatten_to_span(None);
        let _ = RogueGame::spans_to_text(spans);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = rogue_game.flatten_to_span(None);
        let _ = RogueGame::spans_to_text(spans);

        rogue_game.on_tick();
        rogue_game.on_frame();

        let spans = rogue_game.flatten_to_span(None);
        let _ = RogueGame::spans_to_text(spans);

        let elapsed = start_time.elapsed().as_millis();

        println!("UpdateRenderspeed Time taken: {:?}", elapsed);

        #[cfg(not(debug_assertions))]
        assert!(elapsed < 500);
    }
}
