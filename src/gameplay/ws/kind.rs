use bevy::prelude::*;

#[derive(Component)]
pub struct Weapon;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum WeaponKind {
    Orb,
    Lightning,
    Fireball,
}

impl WeaponKind {
    pub fn id(self) -> &'static str {
        match self {
            WeaponKind::Orb => "orb",
            WeaponKind::Lightning => "lightning",
            WeaponKind::Fireball => "fireball",
        }
    }

    pub const ALL: &'static [WeaponKind] =
        &[WeaponKind::Orb, WeaponKind::Lightning, WeaponKind::Fireball];

    pub fn count() -> usize {
        Self::ALL.len()
    }
}
