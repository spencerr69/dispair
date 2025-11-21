use crate::common::weapon::Weapon;

pub trait Poweruppable {
    fn get_next_upgrade(&self) -> Option<DynPowerup>;

    fn upgrade_self(&mut self, powerup: &dyn Powerup);

    fn get_level(&self) -> i32;
}

pub trait Powerup {
    fn upgrade_parent(&self, parent: &mut dyn Poweruppable);

    fn get_name(&self) -> &str;

    fn get_desc(&self) -> &str;

    fn get_new_level(&self) -> i32;

    fn get_current_level(&self) -> i32;
}

pub type DynPowerup = Box<dyn Powerup>;
pub struct WeaponPowerup {
    pub name: String,
    pub desc: String,
    pub new_level: i32,
    pub current_weapon: Option<Box<dyn PoweruppableWeapon>>,
}

pub trait PoweruppableWeapon: Weapon + Poweruppable {}
impl<T> PoweruppableWeapon for T where T: Weapon + Poweruppable {}

impl WeaponPowerup {
    pub fn new(
        name: String,
        desc: String,
        new_level: i32,
        current_weapon: Option<Box<dyn PoweruppableWeapon>>,
    ) -> Self {
        Self {
            name,
            desc,
            new_level,
            current_weapon,
        }
    }
}

impl Powerup for WeaponPowerup {
    fn upgrade_parent(&self, parent: &mut dyn Poweruppable) {
        parent.upgrade_self(self);
    }

    fn get_current_level(&self) -> i32 {
        if let Some(ref current_weapon) = self.current_weapon {
            current_weapon.get_level()
        } else {
            0
        }
    }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_desc(&self) -> &str {
        self.desc.as_str()
    }

    fn get_new_level(&self) -> i32 {
        self.new_level
    }
}
