use std::time::Duration;

use rand::Rng;
use ratatui::style::Style;
use ratatui::style::Stylize;

use crate::character::*;
use crate::coords::Area;
use crate::coords::Direction;
use crate::coords::Position;
use crate::roguegame::*;
use crate::weapon::DamageArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Debuff {
    MarkedForExplosion(u32),
}

impl Debuff {}

pub trait OnDeathEffect {
    fn on_death(&self, enemy: Enemy) -> Option<DamageArea>;
}

impl OnDeathEffect for Debuff {
    fn on_death(&self, enemy: Enemy) -> Option<DamageArea> {
        match self {
            Debuff::MarkedForExplosion(mark_explosion_size) => {
                let area = Area {
                    corner1: Position(
                        enemy.position.0.saturating_sub(*mark_explosion_size as i32),
                        enemy.position.1.saturating_sub(*mark_explosion_size as i32),
                    ),
                    corner2: Position(
                        enemy.position.0.saturating_add(*mark_explosion_size as i32),
                        enemy.position.1.saturating_add(*mark_explosion_size as i32),
                    ),
                };

                Some(DamageArea {
                    damage_amount: enemy.max_health - 1,
                    area,
                    entity: EntityCharacters::AttackBlackout,
                    duration: Duration::from_secs_f64(0.1),
                    blink: true,
                    weapon_stats: None,
                })
            }
        }
    }
}

pub trait EnemyBehaviour {
    fn new(position: Position, damage: i32, health: i32, worth: u32) -> Self;

    fn get_worth(&self) -> u32;

    fn update(&mut self, character: &mut Character, layer: &Layer);
}

#[derive(Clone)]
pub struct Enemy {
    position: Position,
    prev_position: Position,

    pub facing: Direction,

    damage: i32,

    health: i32,
    pub max_health: i32,
    is_alive: bool,

    entitychar: EntityCharacters,

    worth: u32,

    pub debuffs: Vec<Debuff>,
}

pub trait Debuffable {
    fn try_proc(&mut self, debuff: Debuff, chance_to_proc: u32);
}

impl Debuffable for Enemy {
    fn try_proc(&mut self, debuff: Debuff, chance_to_mark: u32) {
        let mut rng = rand::rng();

        let roll = rng.random_range(1..=100);

        match debuff {
            Debuff::MarkedForExplosion(_) => {
                if roll <= chance_to_mark && self.count_debuff(debuff) < 1 {
                    self.debuffs.push(debuff);
                }
            }
        }
    }
}

impl Enemy {
    pub fn count_debuff(&self, debuff: Debuff) -> u32 {
        self.debuffs
            .iter()
            .fold(0, |acc, e| if e == &debuff { acc + 1 } else { acc })
    }

    pub fn explode(&self, mark_explosion_size: u32) -> DamageArea {
        let area = Area {
            corner1: Position(
                self.position.0.saturating_sub(mark_explosion_size as i32),
                self.position.1.saturating_sub(mark_explosion_size as i32),
            ),
            corner2: Position(
                self.position.0.saturating_add(mark_explosion_size as i32),
                self.position.1.saturating_add(mark_explosion_size as i32),
            ),
        };

        DamageArea {
            damage_amount: self.max_health - 1,
            area,
            entity: EntityCharacters::AttackBlackout,
            duration: Duration::from_secs_f64(0.1),
            blink: true,
            weapon_stats: None,
        }
    }
}

impl EnemyBehaviour for Enemy {
    fn new(position: Position, damage: i32, health: i32, worth: u32) -> Self {
        Enemy {
            position: position.clone(),
            prev_position: position,

            facing: Direction::UP,

            damage,

            health,
            max_health: health,
            is_alive: true,

            entitychar: EntityCharacters::Enemy(Style::default()),

            worth,

            debuffs: Vec::new(),
        }
    }

    fn get_worth(&self) -> u32 {
        self.worth.clone()
    }

    fn update(&mut self, character: &mut Character, layer: &Layer) {
        self.prev_position = self.position.clone();

        if is_next_to_character(character.get_pos(), &self.position) {
            character.take_damage(self.damage);
        }

        let (dist_x, dist_y) = self.position.get_distance(character.get_pos());
        let (x, y) = self.position.get();
        let desired_pos: Position;
        let desired_facing: Direction;

        if dist_x.abs() > dist_y.abs() {
            if dist_x > 0 {
                desired_pos = Position::new(x + 1, y);
                desired_facing = Direction::RIGHT;
            } else {
                desired_pos = Position::new(x - 1, y);
                desired_facing = Direction::LEFT;
            }
        } else {
            if dist_y > 0 {
                desired_pos = Position::new(x, y + 1);
                desired_facing = Direction::DOWN;
            } else {
                desired_pos = Position::new(x, y - 1);
                desired_facing = Direction::UP;
            }
        }

        if can_stand(layer, &desired_pos) {
            self.move_to(desired_pos, desired_facing);
        }
    }
}

impl Movable for Enemy {
    fn get_facing(&self) -> Direction {
        self.facing.clone()
    }

    fn get_pos(&self) -> &Position {
        &self.position
    }

    fn get_prev_pos(&self) -> &Position {
        &self.prev_position
    }

    fn move_to(&mut self, new_pos: Position, facing: Direction) {
        self.facing = facing;
        self.set_pos(new_pos);
    }

    fn set_pos(&mut self, new_pos: Position) {
        self.prev_position = self.position.clone();
        self.position = new_pos;
    }

    fn get_entity_char(&self) -> EntityCharacters {
        self.entitychar.clone()
    }
}

impl Damageable for Enemy {
    fn die(&mut self) {
        self.is_alive = false;
    }

    fn get_health(&self) -> &i32 {
        &self.health
    }

    fn is_alive(&self) -> bool {
        self.is_alive.clone()
    }

    fn take_damage(&mut self, damage: i32) {
        let normal_style = Style::default();
        let hurt_style = Style::default().gray().italic();

        self.health -= damage;

        if self.health >= self.max_health / 2 {
            self.entitychar
                .replace(EntityCharacters::Enemy(normal_style));
        }
        if self.health < self.max_health / 2 {
            self.entitychar.replace(EntityCharacters::Enemy(hurt_style));
        }
        if self.health <= 0 {
            self.die();
        }
    }
}
