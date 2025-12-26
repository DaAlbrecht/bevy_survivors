use bevy::prelude::*;

pub(crate) mod assets;
mod behaviours;
pub(crate) mod components;
mod kind;
mod systems;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, behaviours::plugin, systems::plugin));
}

pub(crate) trait ApplySpec {
    fn apply(&self, commands: &mut Commands, entity: Entity);
}

pub mod prelude {
    pub use super::kind::WeaponKind;

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
        spec::{AtlasAnimation, AttackSpec, HitSpec, VisualSpec, WeaponSfx, WeaponSpec},
    };
}
