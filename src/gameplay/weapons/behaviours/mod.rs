use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;

use crate::gameplay::weapons::assets::spec::VisualSpec;

pub mod chain;
pub mod falling;
pub mod homing;
pub mod nova;
pub mod orbiters;
pub mod shared;
pub mod shot;
pub mod zone;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        shot::plugin,
        orbiters::plugin,
        chain::plugin,
        nova::plugin,
        homing::plugin,
        falling::plugin,
        zone::plugin,
        shared::plugin,
    ));
}

#[derive(Component, Clone)]
pub struct WeaponProjectileVisuals(pub VisualSpec);

#[derive(Component, Clone)]
pub struct WeaponImpactVisuals(pub VisualSpec);

#[derive(Component, Clone)]
pub struct WeaponAttackSfx(pub Handle<AudioSample>);

#[derive(Component, Clone)]
pub struct WeaponImpactSfx(pub Handle<AudioSample>);
