use crate::common::powerup::{DynPowerup, PoweruppableWeapon};

pub struct PowerupPopup {
    powerup_choices: Vec<DynPowerup>,
}

impl PowerupPopup {
    pub fn new(current_weapons: &'static Vec<impl PoweruppableWeapon>) -> Self {
        let mut choices = Vec::new();

        current_weapons.iter().for_each(|weapon| {
            let next_upgrade = weapon.get_next_upgrade();
            if let Some(next_upgrade) = next_upgrade {
                choices.push(next_upgrade);
            }
        });

        Self {
            powerup_choices: choices,
        }
    }
}
