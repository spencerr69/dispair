use crate::common::character::{Character, Damageable, Movable, Renderable};
use crate::common::debuffs::{GetDebuffTypes, OnDamageEffect, OnDeathEffect, OnTickEffect};
use crate::common::effects::DamageEffect;
use crate::common::enemies::enemy::{Enemy, EnemyBehaviour, EnemyDrops};
use crate::common::rogue::{Layer, Rogue, get_rand_position_on_edge, is_next_to_character};
use crate::common::timescaler::TimeScaler;
use crate::common::upgrades::upgrade::PlayerState;
use crate::common::weapons::DamageArea;
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
    ) -> Vec<EnemyDrops> {
        if tickcount.is_multiple_of(self.enemy_spawn_ticks) {
            self.spawn_enemy(layer);
        }

        if tickcount.is_multiple_of(self.enemy_move_ticks) {
            self.update_enemies(character, layer, active_damage_effects);
        }

        if tickcount.is_multiple_of(TICK_RATE.floor() as u64) {
            self.scale_enemies();
        }

        self.process_enemy_effects(layer, active_damage_effects, tickcount)
    }

    fn update_enemies(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        active_damage_effects: &mut Vec<DamageEffect>,
    ) {
        self.enemies.borrow_mut().iter_mut().for_each(|enemy| {
            if let Some((desired_pos, desired_facing)) =
                enemy.update(character, layer, active_damage_effects)
                && can_stand(
                    self.player_state.borrow().stats.game_stats.width as i32,
                    self.player_state.borrow().stats.game_stats.height as i32,
                    character,
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

                enemy.move_back(character_stats.shove_amount as i32, layer);
            }
        });
    }

    pub fn spawn_enemy(&mut self, layer: &Layer) {
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

    fn scale_enemies(&mut self) {
        let init_enemy_health = 1.;
        let init_enemy_damage = 1.;
        let init_enemy_spawn_secs =
            Self::DEFAULT_SPAWN_P_S * self.player_state.borrow().stats.game_stats.enemy_spawn_mult;
        let init_enemy_move_secs =
            Self::DEFAULT_MOVE_P_S * self.player_state.borrow().stats.game_stats.enemy_move_mult;
        let init_enemy_gold: u128 = 1;
        let init_enemy_xp: u128 = 1;

        let time_scaler = self.timescaler.borrow().scale_amount;

        self.enemy_health = (init_enemy_health * (time_scaler * 5.).max(1.)).ceil() as i32;

        self.enemy_damage = (init_enemy_damage * (time_scaler / 5.).max(1.)).ceil() as i32;
        self.enemy_spawn_ticks = Rogue::per_sec_to_tick_count(init_enemy_spawn_secs * time_scaler);

        self.enemy_move_ticks =
            Rogue::per_sec_to_tick_count(init_enemy_move_secs * (time_scaler / 6.).max(1.));

        self.enemy_drops = EnemyDrops {
            gold: (init_enemy_gold as f64 * (time_scaler / 2.).max(1.)).ceil() as u128,
            xp: if self.player_state.borrow().upgrade_owned("A") {
                (init_enemy_xp as f64 * (time_scaler / 2.).max(1.)).ceil() as u128
            } else {
                0
            },
        }
    }
}
