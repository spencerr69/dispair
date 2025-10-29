use std::time::{Duration, Instant};

use crate::{
    TICK_RATE,
    character::{Character, Damageable, Movable},
    coords::{Direction, Position},
    effects::DamageEffect,
    enemy::*,
    upgrade::PlayerState,
};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

pub type Layer = Vec<Vec<EntityCharacters>>;

pub struct RogueGame {
    pub player_state: PlayerState,

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
    attack_ticks: u128,

    pub game_over: bool,

    active_damage_effects: Vec<DamageEffect>,

    timer: Duration,
    start_time: Instant,
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
        let enemy_spawn_ticks = Self::per_sec_to_tick_count(1.);

        let start_time = Instant::now();
        let timer = Duration::from_secs(player_state.stats.timer);

        let mut game = RogueGame {
            player_state: player_state.clone(),
            character: Character::new(player_state),
            layer_base: base,
            layer_entities: entities,
            layer_effects: effects,
            height,
            width,
            attack_ticks,
            enemy_move_ticks,
            enemy_spawn_ticks,
            tickcount: 0,
            enemies: vec![],
            game_over: false,
            active_damage_effects: vec![],
            start_time,
            timer,
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
        self.tickcount += 1;

        if self.start_time.elapsed() >= self.timer {
            self.game_over = true;
        }

        if !self.character.is_alive() {
            self.game_over = true;
        }

        self.enemies = self
            .enemies
            .clone()
            .into_iter()
            .filter(|e| {
                if !e.is_alive() {
                    update_entity_positions(&mut self.layer_entities, e);
                    set_entity(
                        &mut self.layer_entities,
                        e.get_pos(),
                        EntityCharacters::Empty,
                    )
                    .unwrap_or(());
                    return false;
                } else {
                    return true;
                };
            })
            .collect();

        if self.tickcount % self.enemy_spawn_ticks == 0 {
            self.spawn_enemy();
        }

        if self.tickcount % self.enemy_move_ticks == 0 {
            self.enemies.iter_mut().for_each(|enemy| {
                enemy.update(&mut self.character, &self.layer_entities);
                update_entity_positions(&mut self.layer_entities, enemy);
            });
            self.change_low_health_enemies_questionable();
        }

        if self.tickcount % self.attack_ticks == 0 {
            let (damage_areas, mut damage_effects) = self.character.attack(&mut self.layer_effects);
            damage_areas.iter().for_each(|area| {
                area.deal_damage(&mut self.enemies);
            });
            self.active_damage_effects.append(&mut damage_effects)
        }
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

        self.change_low_health_enemies_questionable();
    }

    pub fn change_low_health_enemies_questionable(&mut self) {
        self.enemies.iter().for_each(|enemy| {
            update_entity_positions(&mut self.layer_entities, enemy);
        });
    }

    pub fn update_stats(&mut self) {
        self.attack_ticks = (self.attack_ticks as f64 / self.character.attack_speed).ceil() as u128
    }

    pub fn spawn_enemy(&mut self) {
        self.enemies.push(Enemy::new(
            get_rand_position_on_edge(&self.layer_entities),
            1,
            5,
            self.get_enemy_worth(),
        ))
    }

    fn get_enemy_worth(&self) -> u32 {
        1
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') => move_entity(
                &mut self.layer_entities,
                &mut self.character,
                Direction::DOWN,
            ),
            KeyCode::Char('w') => {
                move_entity(&mut self.layer_entities, &mut self.character, Direction::UP)
            }
            KeyCode::Char('d') => move_entity(
                &mut self.layer_entities,
                &mut self.character,
                Direction::RIGHT,
            ),
            KeyCode::Char('a') => move_entity(
                &mut self.layer_entities,
                &mut self.character,
                Direction::LEFT,
            ),
            KeyCode::Esc => self.game_over = true,
            _ => {}
        }
    }

    pub fn init_character(&mut self) {
        let mut rng = rand::rng();

        let (x, y) = (
            rng.random_range(0..self.width) as i32,
            rng.random_range(0..self.height) as i32,
        );

        self.character.set_pos(Position(x, y));
        update_entity_positions(&mut self.layer_entities, &self.character);
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

    pub fn to_text(&self) -> Text<'static> {
        let map = self.flatten_to_span();

        let out: Text<'static> = map
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
}

impl Widget for &RogueGame {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let timer = self.timer.saturating_sub(self.start_time.elapsed());

        let title = Line::from(" dispair.run ".bold());

        let instructions = Line::from(vec![
            " health: ".into(),
            self.character.get_health().to_string().bold(),
            " ".into(),
            " time: ".into(),
            timer.as_secs().to_string().bold().into(),
            " ".into(),
        ]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.right_aligned())
            .border_set(border::THICK);

        Paragraph::new(self.to_text())
            .centered()
            .block(block)
            .render(area, buf);
    }
}

pub fn get_pos<'a>(layer: &'a Layer, position: &Position) -> &'a EntityCharacters {
    let (x, y) = position.get_as_usize();
    &layer[y][x]
}
// pub fn get_pos_mut<'a>(layer: &'a mut Layer, position: &Position) -> &'a mut EntityCharacters {
//     let (x, y) = position.get_as_usize();
//     &mut layer[y][x]
// }

pub fn update_entity_positions(layer: &mut Layer, entity: &impl Movable) {
    set_entity(layer, entity.get_prev_pos(), EntityCharacters::Empty).unwrap_or(());
    set_entity(layer, entity.get_pos(), entity.get_entity_char()).unwrap_or(());
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
        update_entity_positions(layer, entity);
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

pub fn is_next_to_character(layer: &Layer, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    let height = layer.len();
    let width = if height > 0 { layer[0].len() } else { 0 };

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dy == 0 && dx == 0 {
                continue;
            }

            let new_y = y as isize + dy;
            let new_x = x as isize + dx;

            if new_y >= 0 && new_y < height as isize && new_x >= 0 && new_x < width as isize {
                if layer[new_y as usize][new_x as usize].is_char() {
                    return true;
                }
            }
        }
    }
    false
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
