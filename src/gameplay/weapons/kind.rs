use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component)]
pub struct Weapon;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Deserialize)]
pub enum WeaponKind {
    Aoe,
    Orb,
    Lightning,
    Fireball,
    Energy,
    Scale,
    Circles,
    Icelance,
    Hammer,
    Slash,
    Sword,
}

impl WeaponKind {
    pub const ALL: &'static [WeaponKind] = &[
        WeaponKind::Aoe,
        WeaponKind::Orb,
        WeaponKind::Lightning,
        WeaponKind::Fireball,
        WeaponKind::Energy,
        WeaponKind::Scale,
        WeaponKind::Circles,
        WeaponKind::Icelance,
        WeaponKind::Hammer,
        WeaponKind::Slash,
        WeaponKind::Sword,
        // TODO: WeaponKind::Thorn,
    ];
}
