use std::time::{Duration, Instant};

use crate::{
    TICK_RATE,
    carnagereport::CarnageReport,
    center,
    character::{Character, Damageable, Movable},
    coords::{Direction, Position},
    effects::DamageEffect,
    enemy::*,
    timescaler::TimeScaler,
    upgrade::PlayerState,
};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    Frame,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

pub type Layer = Vec<Vec<EntityCharacters>>;

pub struct RogueGame {
    pub player_state: PlayerState,

    pub carnage_report: Option<CarnageReport>,

    pub map_text: Text<'static>,

    character: Character,
    layer_base: Layer,
    layer_entities: Layer,
    layer_effects: Layer,

    tickcount: u128,

    height: usize,
    width: usize,

    enemies: Vec<Enemy>,

    enemy_spawn_ticks: u128,
    enemy_move_ticks: u128,

    enemy_health: i32,
    enemy_damage: i32,
    enemy_worth: u32,

    attack_ticks: u128,

    pub game_over: bool,
    pub exit: bool,

    active_damage_effects: Vec<DamageEffect>,

    timer: Duration,
    start_time: Instant,

    timescaler: TimeScaler,
}

impl RogueGame {
    pub fn new(player_state: PlayerState) -> Self {
        let width = player_state.stats.width;
        let height = player_state.stats.height;

        let mut base: Layer = Vec::from(Vec::new());
        let mut entities: Layer = Vec::from(Vec::new());
        let mut effects: Layer = Vec::from(Vec::new());

        let mut rng = rand::rng();

        for _ in 0..height {
            let mut baseline = Vec::new();
            let mut entityline = Vec::new();
            let mut effectsline = Vec::new();
            for _ in 0..width {
                let choice = rng.random_range(0..=1);
                match choice {
                    0 => baseline.push(EntityCharacters::Background1),
                    _ => baseline.push(EntityCharacters::Background2),
                }
                entityline.push(EntityCharacters::Empty);
                effectsline.push(EntityCharacters::Empty);
            }
            base.push(baseline);
            entities.push(entityline);
            effects.push(effectsline);
        }

        let attack_ticks = Self::per_sec_to_tick_count(1.5);
        let enemy_move_ticks = Self::per_sec_to_tick_count(2.);
        let enemy_spawn_ticks =
            Self::per_sec_to_tick_count(0.4 * player_state.stats.enemy_spawn_mult);

        let start_time = Instant::now();
        let timer = Duration::from_secs(player_state.stats.timer);

        let mut game = RogueGame {
            player_state: player_state.clone(),
            character: Character::new(player_state.clone()),
            layer_base: base,
            layer_entities: entities,
            layer_effects: effects,
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
            game_over: false,
            active_damage_effects: vec![],
            start_time,
            timer,
            timescaler: TimeScaler::now().offset_start_time(player_state.stats.time_offset),
        };

        game.init_character();
        game.update_stats();

        game
    }

    pub fn per_sec_to_tick_count(per_sec: f64) -> u128 {
        let per_tick = TICK_RATE / per_sec;
        per_tick.ceil() as u128
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
                    // update_entity_positions(&mut self.layer_entities, e);
                    // set_entity(
                    //     &mut self.layer_entities,
                    //     e.get_pos(),
                    //     EntityCharacters::Empty,
                    // )
                    // .unwrap_or(());
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

                        let damage_effect = DamageEffect::new(damage_area);

                        self.active_damage_effects.push(damage_effect);
                    }
                });
        });

        if self.tickcount % self.enemy_spawn_ticks == 0 {
            self.spawn_enemy();
        }

        if self.tickcount % self.enemy_move_ticks == 0 {
            self.enemies.iter_mut().for_each(|enemy| {
                enemy.update(&mut self.character, &self.layer_entities);
                // update_entity_positions(&mut self.layer_entities, enemy);

                if self.player_state.stats.shove_amount > 0
                    && is_next_to_character(self.character.get_pos(), enemy.get_prev_pos())
                {
                    if self.player_state.stats.shove_damage > 0 {
                        enemy.take_damage(
                            (self.player_state.stats.shove_damage as f64
                                * self.player_state.stats.damage_mult)
                                .ceil() as i32,
                        );
                    }

                    enemy.move_back(
                        self.player_state.stats.shove_amount as i32,
                        &self.layer_entities,
                    );
                }
            });
            // self.change_low_health_enemies_questionable();
        }

        if self.tickcount % TICK_RATE.floor() as u128 == 0 {
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

        update_layer(&mut self.layer_entities, &self.enemies, &self.character);
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

        let spans = self.flatten_to_span();

        self.map_text = Self::spans_to_text(spans);
    }

    // pub fn change_low_health_enemies_questionable(&mut self) {
    //     self.enemies.iter().for_each(|enemy| {
    //         update_entity_positions(&mut self.layer_entities, enemy);
    //     });
    // }

    pub fn update_stats(&mut self) {
        self.attack_ticks = (self.attack_ticks as f64 / self.character.attack_speed).ceil() as u128;
    }

    fn scale_enemies(&mut self) {
        let init_enemy_health = 3.;
        let init_enemy_damage = 1.;
        let init_enemy_spawn_secs = 0.4 * self.player_state.stats.enemy_spawn_mult;
        let init_enemy_move_secs = 2. * self.player_state.stats.enemy_move_mult;
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
        // update_entity_positions(&mut self.layer_entities, &self.character);
    }

    pub fn flatten_to_span(&self) -> Vec<Vec<Span<'static>>> {
        let mut out: Vec<Vec<Span<'static>>> = self
            .layer_base
            .iter()
            .map(|line| line.iter().map(|entity| entity.to_styled()).collect())
            .collect();

        for (y, row) in out.iter_mut().enumerate() {
            for (x, pos) in row.iter_mut().enumerate() {
                if self.layer_effects[y][x] != EntityCharacters::Empty
                    || self.layer_entities[y][x] != EntityCharacters::Empty
                {
                    if self.layer_effects[y][x] != EntityCharacters::Empty {
                        pos.clone_from(&self.layer_effects[y][x].to_styled());
                    } else {
                        pos.clone_from(&self.layer_entities[y][x].to_styled());
                    }
                }
            }
        }

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

    pub fn render(&self, frame: &mut Frame) {
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

        let content_area = center(
            block.inner(frame.area()),
            self.layer_base[0].len() as u16,
            self.layer_base.len() as u16,
        );

        let content = Paragraph::new(self.map_text.clone()).centered();

        frame.render_widget(block, frame.area());
        frame.render_widget(content, content_area);

        if let Some(ref mut carnage) = self.carnage_report.clone() {
            carnage.render(frame);
        }
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

pub fn update_layer(layer_entities: &mut Layer, enemies: &Vec<Enemy>, character: &Character) {
    clear_layer(layer_entities);

    enemies.iter().for_each(|enemy| {
        let (x, y) = enemy.get_pos().get_as_usize();

        layer_entities[y][x] = enemy.get_entity_char();
    });

    let (char_x, char_y) = character.get_pos().get_as_usize();
    layer_entities[char_y][char_x] = character.get_entity_char();
}

// pub fn update_entity_positions(layer: &mut Layer, entity: &impl Movable) {
//     set_entity(layer, entity.get_prev_pos(), EntityCharacters::Empty).unwrap_or(());
//     set_entity(layer, entity.get_pos(), entity.get_entity_char()).unwrap_or(());
// }

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
    AttackBlackout,
}

//hurt style .gray().italic()

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
            EntityCharacters::AttackBlackout => {
                Span::from(ratatui::symbols::block::FULL).bold().white()
            }
        }
    }

    pub fn replace(&mut self, new_entity: EntityCharacters) {
        *self = new_entity;
    }

    pub fn is_char(&self) -> bool {
        match *self {
            EntityCharacters::Character(_) => true,
            _ => false,
        }
    }
}
