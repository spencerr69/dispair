use crate::common::{
    charms::CharmWrapper,
    weapons::{Weapon, WeaponWrapper},
};

pub trait Poweruppable {
    fn get_max_level(&self) -> i32 {
        5
    }

    fn get_next_upgrade(&self, levels_up: i32) -> Option<DynPowerup> {
        if self.get_level() >= self.get_max_level() {
            None
        } else {
            let new_level = self.get_level() + levels_up;

            Some(Box::new(PowerupUpgrade::new(
                &self.get_name(),
                self.upgrade_desc(new_level),
                self.get_level(),
                new_level,
                self.get_powerup_type(),
            )))
        }
    }

    fn get_name(&self) -> String;

    fn get_powerup_type(&self) -> PowerupTypes;

    fn upgrade_desc(&self, level: i32) -> String;

    fn upgrade_self(&mut self, powerup: &DynPowerup);

    fn get_level(&self) -> i32;
}

pub trait Powerup {
    fn get_name(&self) -> &str;

    fn get_desc(&self) -> &str;

    fn get_powerup_type(&self) -> PowerupTypes;

    fn get_new_level(&self) -> i32;

    fn get_current_level(&self) -> i32;
}

pub type DynPowerup = Box<dyn Powerup>;
pub struct PowerupUpgrade {
    pub name: String,
    pub desc: String,
    pub new_level: i32,
    pub curr_level: i32,
    pub powerup_type: PowerupTypes,
}

pub trait PoweruppableWeapon: Weapon + Poweruppable {}
impl<T> PoweruppableWeapon for T where T: Weapon + Poweruppable {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PowerupTypes {
    Weapon,
    Charm,
}

impl PowerupUpgrade {
    #[must_use]
    pub fn new(
        name: &str,
        desc: String,
        curr_level: i32,
        new_level: i32,
        powerup_type: PowerupTypes,
    ) -> Self {
        Self {
            name: name.to_uppercase(),
            desc,
            curr_level,
            new_level,
            powerup_type,
        }
    }

    #[must_use]
    pub fn init_weapon(wrapper: WeaponWrapper) -> Self {
        let weapon_name: &'static str = wrapper.into();
        let upper = weapon_name.to_uppercase();
        Self::new(
            &upper,
            format!("New METHOD: {upper}"),
            0,
            1,
            PowerupTypes::Weapon,
        )
    }

    #[must_use]
    pub fn init_charm(wrapper: CharmWrapper) -> Self {
        let charm_name: &'static str = wrapper.into();
        let upper = charm_name.to_uppercase();
        Self::new(
            &upper,
            format!("New CHARM: {upper}"),
            0,
            1,
            PowerupTypes::Charm,
        )
    }
}

impl Powerup for PowerupUpgrade {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_desc(&self) -> &str {
        self.desc.as_str()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        self.powerup_type
    }

    fn get_new_level(&self) -> i32 {
        self.new_level
    }

    fn get_current_level(&self) -> i32 {
        self.curr_level
    }
}
