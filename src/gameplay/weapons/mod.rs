use bevy::prelude::*;

pub(crate) mod assets;
mod behaviours;
pub(crate) mod components;
mod kind;
mod runtime;
mod systems;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, behaviours::plugin, systems::plugin));
}

pub mod prelude {
    pub use super::kind::{Weapon, WeaponKind};

    pub use super::components::*;

    pub use super::systems::{
        attack::WeaponAttackEvent, cooldown::WeaponCooldown, hit::WeaponHitEvent,
        pickup::PickUpWeaponEvent,
    };

    pub use super::behaviours::{
        chain::ChainSpec, falling::FallingSpec, homing::HomingSpec, nova::NovaSpec,
        orbiters::OrbitersSpec, shot::ShotSpec, zone::ZoneSpec,
    };

    pub use super::assets::{
        WeaponMap,
        spec::{AtlasAnim, AttackSpec, HitSpec, VisualSpec, WeaponSfx, WeaponSpec},
    };

    pub use super::runtime::{
        ApplySpec,
        visuals::{WeaponImpactVisuals, WeaponProjectileVisuals},
    };
}
