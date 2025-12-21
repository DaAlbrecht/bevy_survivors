use bevy::prelude::*;

pub(crate) mod assets;
mod behaviours;
mod kind;
mod runtime;
mod systems;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, behaviours::plugin, systems::plugin));
}

pub mod prelude {
    pub use super::kind::{Weapon, WeaponKind};

    pub use super::systems::{
        cooldown::WeaponCooldown,
        pickup::{PickUpWeaponEvent, UpgradeWeaponEvent},
    };

    pub use super::behaviours::{
        chain::ChainLightningSpec, orbiters::OrbitersSpec, shared::*, shot::ShotSpec,
    };

    pub use super::assets::{
        WeaponAssets,
        spec::{AtlasAnim, AttackSpec, HitSpec, VisualSpec, WeaponSfx, WeaponSpec},
    };

    pub use super::runtime::ApplySpec;
}
