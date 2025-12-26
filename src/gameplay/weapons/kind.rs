use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Deserialize)]
pub enum WeaponKind {
    Orb,
    Lightning,
    Fireball,
    Energy,
    Scale,
    Circles,
    Icelance,
    LightningBeam,
    DragonBreath,
    Hammer, // Thorn,
}

impl WeaponKind {
    pub const ALL: &'static [WeaponKind] = &[
        WeaponKind::Orb,
        WeaponKind::Lightning,
        WeaponKind::Fireball,
        WeaponKind::Energy,
        WeaponKind::Scale,
        WeaponKind::Circles,
        WeaponKind::Icelance,
        WeaponKind::Hammer,
        WeaponKind::LightningBeam,
        WeaponKind::DragonBreath,
        // TODO: WeaponKind::Thorn,
    ];
}
