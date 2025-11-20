use crate::common::weapon::Weapon;

pub struct WeaponPowerup<'a> {
    pub name: String,
    pub desc: String,
    pub new_level: i32,
    pub current_weapon: Option<&'a Box<dyn Weapon>>,
}

impl<'a> WeaponPowerup<'a> {
    pub fn new(
        name: String,
        desc: String,
        new_level: i32,
        current_weapon: Option<&'a Box<dyn Weapon>>,
    ) -> Self {
        Self {
            name,
            desc,
            new_level,
            current_weapon,
        }
    }
}

impl Powerup for WeaponPowerup<'_> {
    fn upgrade_parent(&self, parent: &mut dyn Poweruppable) {
        parent.upgrade_self();
    }
}

pub trait Poweruppable {
    fn get_next_upgrade(&self) -> Option<DynPowerup>;

    fn upgrade_self(&mut self);
}

pub trait Powerup {
    fn upgrade_parent(&self, parent: &mut dyn Poweruppable);
}

pub type DynPowerup = Box<dyn Powerup>;
