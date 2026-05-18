use crate::common::PlayerStateRef;
use crate::common::character::Renderable;
use crate::common::coords::AreaWrapper::Square;
use crate::common::coords::{Position, SquareArea};
use crate::common::effects::DamageEffect;
use crate::common::entities::EntityCharacters;
use crate::common::pickups::Pickupable;
use crate::common::pickups::poweruporb::PowerupOrb;
use crate::common::pickups::{PickupEffect, PickupTypes};
use crate::common::rogue::Layer;
use crate::common::utils::get_rand_position_on_layer;
use crate::prelude::Duration;
use ratatui::prelude::Style;

pub struct PickupWrangler {
    pub player_state: PlayerStateRef,
    pub pickups: Vec<PickupTypes>,
    pub start_popup: bool,
}

impl PickupWrangler {
    pub fn new(player_state: PlayerStateRef) -> Self {
        PickupWrangler {
            player_state,
            start_popup: false,
            pickups: Vec::new(),
        }
    }

    pub fn spawn_orb(&mut self, layer: &Layer) {
        if !self.player_state.borrow().upgrade_owned("A") {
            let position = get_rand_position_on_layer(layer);

            self.pickups
                .push(PickupTypes::PowerupOrb(PowerupOrb::new(position)));
        }
    }

    pub fn handle_pickups(
        &mut self,
        char_pos: &Position,
        active_damage_effects: &mut Vec<DamageEffect>,
    ) {
        self.pickups.iter_mut().for_each(|pickup| {
            if pickup.get_inner().get_pos() == char_pos {
                let effect = pickup.get_inner_mut().on_pickup();

                match effect {
                    PickupEffect::PowerupOrb => {
                        let area = SquareArea::new(
                            Position(0, 0),
                            Position(
                                self.player_state.borrow().stats.game_stats.width as i32,
                                self.player_state.borrow().stats.game_stats.height as i32,
                            ),
                        );

                        active_damage_effects.push(DamageEffect::new(
                            Square(area),
                            EntityCharacters::AttackWeak(Style::new().red()),
                            Duration::from_secs_f64(0.5),
                            false,
                        ));

                        self.start_popup = true;
                    }
                }
            }
        });

        self.pickups
            .retain(|pickup| !pickup.get_inner().is_picked_up());
    }

    pub fn on_tick(
        &mut self,
        tickcount: u64,
        char_pos: &Position,
        active_damage_effects: &mut Vec<DamageEffect>,
    ) {
        self.handle_pickups(char_pos, active_damage_effects);

        self.pickups
            .iter_mut()
            .for_each(|pickup| pickup.get_inner_mut().animate(tickcount % 1000));
    }
}
