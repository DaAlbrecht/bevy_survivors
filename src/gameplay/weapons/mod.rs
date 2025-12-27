use bevy::prelude::*;

mod behaviours;
pub(crate) mod components;
mod kind;
pub(crate) mod spec;
mod systems;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((spec::plugin, behaviours::plugin, systems::plugin));
}

pub mod prelude {
    pub use super::kind::{Weapon, WeaponKind};

    pub use super::components::*;

    pub use super::systems::{
        attack::WeaponAttackEvent, cooldown::WeaponCooldown, hit::WeaponHitEvent,
        pickup::PickUpWeaponEvent,
    };

    pub use super::behaviours::{
        chain::ChainSpec, falling::FallingSpec, homing::HomingSpec, melee::MeleeSpec,
        nova::NovaSpec, orbiters::OrbitersSpec, shot::ShotSpec, zone::ZoneSpec,
    };

    pub use super::spec::{
        WeaponMap,
        apply::{
            ApplySpec,
            visuals::{WeaponImpactVisuals, WeaponProjectileVisuals},
        },
        components::{AtlasAnim, AttackSpec, HitSpec, VisualSpec, WeaponSfx, WeaponSpec},
    };
}
