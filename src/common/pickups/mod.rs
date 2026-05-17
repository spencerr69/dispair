use crate::common::character::Renderable;
use crate::common::coords::Position;
use crate::common::entities::EntityCharacters;
use crate::common::pickups::poweruporb::PowerupOrb;

pub mod pickupwrangler;
pub mod poweruporb;

pub trait Pickupable: Renderable {
    /// Animates the pickup based on the current game tick.
    fn animate(&mut self, tick: u64);

    /// sets `picked_up` to true and returns pickupeffect
    fn on_pickup(&mut self) -> PickupEffect;

    fn is_picked_up(&self) -> bool;
}

pub enum PickupTypes {
    PowerupOrb(PowerupOrb),
}

impl PickupTypes {
    #[must_use]
    pub fn get_inner(&self) -> &impl Pickupable {
        match self {
            PickupTypes::PowerupOrb(orb) => orb,
        }
    }

    #[must_use]
    pub fn get_inner_mut(&mut self) -> &mut impl Pickupable {
        match self {
            PickupTypes::PowerupOrb(orb) => orb,
        }
    }
}

impl Renderable for PickupTypes {
    fn get_pos(&self) -> &Position {
        self.get_inner().get_pos()
    }

    fn get_entity_char(&self) -> &EntityCharacters {
        self.get_inner().get_entity_char()
    }
}

#[derive(Debug, Clone)]
pub enum PickupEffect {
    PowerupOrb,
}
