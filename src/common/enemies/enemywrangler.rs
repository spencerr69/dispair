use crate::common::character::{Character, Damageable, Movable, Renderable};
use crate::common::coords::{ChaosArea, Position};
use crate::common::debuffs::{GetDebuffTypes, OnDamageEffect, OnDeathEffect, OnTickEffect};
use crate::common::effects::DamageEffect;
use crate::common::enemies::enemy::{Enemy, EnemyBehaviour, EnemyDrops};
use crate::common::map::Layer;
use crate::common::timescaler::TimeScaler;
use crate::common::utils::{
    can_stand, convert_range, get_rand_position_on_edge, is_next_to_character,
    per_sec_to_tick_count, per_sec_to_tick_count_to_u64,
};
use crate::common::weapons::DamageArea;
use crate::common::{PlayerStateRef, TICK_RATE};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EnemyWrangler {
    pub enemies: Rc<RefCell<Vec<Enemy>>>,
    pub enemy_spawn_ticks: u64,
    pub enemy_spawn_mult: f64,
    pub enemy_move_ticks: u64,
    pub enemy_health: i32,
    pub enemy_damage: i32,
    pub enemy_drops: EnemyDrops,
    pub player_state: PlayerStateRef,
    pub timescaler: Rc<RefCell<TimeScaler>>,
}

impl EnemyWrangler {
    const ENEMY_CAP: u64 = 2000;
    const DEFAULT_SPAWN_P_S: f64 = 0.4;
    const DEFAULT_MOVE_P_S: f64 = 1.3;
    const DEFAULT_HEALTH: i32 = 2;

    pub fn new(
        player_state: PlayerStateRef,
        timescaler: Rc<RefCell<TimeScaler>>,
        enemies: Rc<RefCell<Vec<Enemy>>>,
    ) -> Self {
        let player_state_ref = player_state.borrow().clone();

        let enemy_move_ticks = per_sec_to_tick_count_to_u64(Self::DEFAULT_MOVE_P_S);
        let enemy_spawn_ticks = per_sec_to_tick_count_to_u64(
            Self::DEFAULT_SPAWN_P_S * player_state_ref.stats.game_stats.enemy_spawn_mult,
        );

        Self {
            enemy_move_ticks,
            enemy_spawn_ticks,
            enemy_spawn_mult: 1.0,
            enemy_damage: 1,
            enemy_health: Self::DEFAULT_HEALTH,
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
    ) -> Vec<EnemyDrops> {
        if tickcount.is_multiple_of(self.enemy_spawn_ticks) {
            for _ in 0..self.enemy_spawn_mult.ceil() as i32 {
                self.spawn_enemy(layer);
            }
        }

        if tickcount.is_multiple_of(self.enemy_move_ticks) {
            self.update_enemies(character, layer, active_damage_effects);
        }

        if tickcount.is_multiple_of(TICK_RATE.floor() as u64) {
            self.scale_enemies();
        }

        self.process_enemy_effects(layer, active_damage_effects, tickcount)
    }

    #[must_use]
    pub fn get_spawn_multiplier(&self) -> f64 {
        if self.enemy_spawn_ticks > 1 {
            convert_range(self.enemy_spawn_ticks as f64, 50., 1., 0., 1.)
        } else {
            convert_range(self.enemy_spawn_mult, 1., 10., 1., 100.)
        }
    }

    fn update_enemies(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        active_damage_effects: &mut Vec<DamageEffect>,
    ) {
        let enemy_area = ChaosArea::new(self.get_enemy_positions());

        self.enemies.borrow_mut().iter_mut().for_each(|enemy| {
            if let Some((desired_pos, desired_facing)) =
                enemy.update(character, layer, active_damage_effects)
                && can_stand(
                    self.player_state.borrow().stats.game_stats.width as i32,
                    self.player_state.borrow().stats.game_stats.height as i32,
                    Some(character),
                    &desired_pos,
                )
                && !desired_pos.is_in_area(&enemy_area)
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

                enemy.move_back(character_stats.shove_amount as i32, layer);
            }
        });
    }

    pub fn spawn_enemy(&mut self, layer: &Layer) {
        if self.enemies.borrow().len() as u64 >= Self::ENEMY_CAP {
            return;
        }

        let position = get_rand_position_on_edge(layer);

        let enemy_area = ChaosArea::new(self.get_enemy_positions());

        if position.is_in_area(&enemy_area) {
            return;
        }

        self.enemies.borrow_mut().push(Enemy::new(
            get_rand_position_on_edge(layer),
            self.enemy_damage,
            self.enemy_health,
            self.enemy_drops.clone(),
        ));
    }

    fn process_enemy_effects(
        &mut self,
        layer: &Layer,
        active_damage_effects: &mut Vec<DamageEffect>,
        tickcount: u64,
    ) -> Vec<EnemyDrops> {
        let mut damage_areas: Vec<DamageArea> = Vec::new();

        let mut drops = Vec::new();

        let mut enemies = self.enemies.borrow_mut().clone();

        for enemy in &mut enemies {
            let mut debuffs = enemy.debuffs.clone();

            for debuff in &mut debuffs {
                if let Some(damage_area) = debuff.on_tick(enemy, layer, tickcount) {
                    damage_areas.push(damage_area);
                }
                if let Some(damage_area) = debuff.on_damage(enemy, layer, &self.enemies.borrow()) {
                    {
                        damage_areas.push(damage_area);
                    }
                }
            }

            debuffs.retain(|d| !d.complete);
            enemy.debuffs = debuffs;

            if !enemy.is_alive() {
                if !enemy.debuffs.get_on_death_effects().is_empty() {
                    for debuff in &enemy.debuffs {
                        if let Some(damage_area) = debuff.on_death(enemy, layer) {
                            damage_areas.push(damage_area);
                        }
                    }
                }

                drops.push(enemy.get_drops());
            }
        }

        self.enemies.replace(enemies);

        self.enemies.borrow_mut().retain(Damageable::is_alive);

        for damage_area in damage_areas {
            damage_area.deal_damage(&mut self.enemies.borrow_mut());

            let damage_effect = DamageEffect::from(damage_area);

            active_damage_effects.push(damage_effect);
        }

        drops
    }

    pub fn on_frame(&mut self) {
        self.enemies.borrow_mut().iter_mut().for_each(|e| {
            e.change_style_with_debuff();
        });
    }

    fn scale_enemies(&mut self) {
        let init_enemy_health = Self::DEFAULT_HEALTH;
        let init_enemy_damage = 1.;
        let init_enemy_spawn_secs =
            Self::DEFAULT_SPAWN_P_S * self.player_state.borrow().stats.game_stats.enemy_spawn_mult;
        let init_enemy_move_secs =
            Self::DEFAULT_MOVE_P_S * self.player_state.borrow().stats.game_stats.enemy_move_mult;
        let init_enemy_gold: u128 = 1;
        let init_enemy_xp: u128 = 1;

        let time_scaler = self.timescaler.borrow().doom;

        self.enemy_health =
            (f64::from(init_enemy_health) * (time_scaler * 0.5).max(1.)).ceil() as i32;

        self.enemy_damage = (init_enemy_damage * (time_scaler / 50.).max(1.)).ceil() as i32;
        let enemy_spawn_calc =
            per_sec_to_tick_count(init_enemy_spawn_secs * (0.3 * time_scaler).max(1.));
        if enemy_spawn_calc > 1.0 {
            self.enemy_spawn_ticks = enemy_spawn_calc.ceil() as u64;
        } else {
            self.enemy_spawn_ticks = 1;
            self.enemy_spawn_mult = convert_range(enemy_spawn_calc, 1.0, 0.0, 1.0, 5.0);
        }

        self.enemy_move_ticks =
            per_sec_to_tick_count_to_u64(init_enemy_move_secs * (time_scaler / 9.).max(1.));

        if self.enemy_spawn_ticks < 5 {
            self.enemy_spawn_ticks =
                per_sec_to_tick_count_to_u64(init_enemy_move_secs * (time_scaler / 18.).max(1.))
                    .min(5);
        }

        self.enemy_drops = EnemyDrops {
            gold: (init_enemy_gold as f64 * (time_scaler / 2.).max(1.)).ceil() as u128,
            xp: if self.player_state.borrow().upgrade_owned("A") {
                (init_enemy_xp as f64 * (time_scaler / 2.).max(1.)).ceil() as u128
            } else {
                0
            },
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
}
