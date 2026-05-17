use crate::common::character::{Character, Damageable, Movable, Renderable};
use crate::common::coords::Position;
use crate::common::effects::DamageEffect;
use crate::common::enemies::enemy::{Enemy, EnemyBehaviour, EnemyDrops};
use crate::common::rogue::{Layer, Rogue, is_next_to_character};
use crate::common::timescaler::TimeScaler;
use crate::common::upgrades::upgrade::PlayerState;
use crate::common::{TICK_RATE, can_stand};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EnemyWrangler {
    enemies: Rc<RefCell<Vec<Enemy>>>,
    enemy_spawn_ticks: u64,
    enemy_move_ticks: u64,
    enemy_health: i32,
    enemy_damage: i32,
    enemy_drops: EnemyDrops,
    player_state: Rc<RefCell<PlayerState>>,
    timescaler: Rc<RefCell<TimeScaler>>,
}

impl EnemyWrangler {
    const DEFAULT_SPAWN_P_S: f64 = 0.4;
    const DEFAULT_MOVE_P_S: f64 = 2.;

    pub fn new(
        player_state: Rc<RefCell<PlayerState>>,
        timescaler: Rc<RefCell<TimeScaler>>,
        enemies: Rc<RefCell<Vec<Enemy>>>,
    ) -> Self {
        let player_state_ref = player_state.borrow().clone();

        let enemy_move_ticks = Rogue::per_sec_to_tick_count(Self::DEFAULT_MOVE_P_S);
        let enemy_spawn_ticks = Rogue::per_sec_to_tick_count(
            Self::DEFAULT_SPAWN_P_S * player_state_ref.stats.game_stats.enemy_spawn_mult,
        );

        Self {
            enemy_move_ticks,
            enemy_spawn_ticks,
            enemy_damage: 1,
            enemy_health: 3,
            enemy_drops: EnemyDrops { gold: 1, xp: 0 },
            enemies,
            player_state,
            timescaler,
        }
    }

    pub fn on_tick(
        &mut self,
        tickcount: u64,
        character: &mut Character,
        layer: &Layer,
        active_damage_effects: &mut Vec<DamageEffect>,
    ) {
        if tickcount.is_multiple_of(self.enemy_spawn_ticks) {
            self.spawn_enemy(character);
        }

        if tickcount.is_multiple_of(self.enemy_move_ticks) {
            self.update_enemies(character, layer, active_damage_effects);
        }

        if tickcount.is_multiple_of(TICK_RATE.floor() as u64) {
            self.scale_enemies();
        }
    }

    fn update_enemies(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        active_damage_effects: &mut Vec<DamageEffect>,
    ) {
        self.enemies.borrow_mut().iter_mut().for_each(|mut enemy| {
            if let Some((desired_pos, desired_facing)) =
                enemy.update(character, layer, active_damage_effects)
                && can_stand(
                    self.player_state.borrow().stats.game_stats.width as i32,
                    self.player_state.borrow().stats.game_stats.height as i32,
                    &character,
                    &desired_pos,
                )
            {
                enemy.move_to(desired_pos, desired_facing);
            }

            let character_stats = &self.player_state.borrow().stats.player_stats;

            if character_stats.shove_amount > 0
                && is_next_to_character(character.get_pos(), enemy.get_prev_pos())
            {
                if character_stats.shove_damage > 0 {
                    enemy.take_damage(
                        (f64::from(character_stats.shove_damage) * character_stats.damage_mult)
                            .ceil() as i32,
                    );
                }

                enemy.move_back(character_stats.shove_amount as i32, &layer);
            }
        })
    }
}
