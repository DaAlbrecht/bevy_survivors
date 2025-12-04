use bevy::prelude::*;

use crate::gameplay::weapons::WeaponType;

pub(crate) mod abilities;
pub(crate) mod character_controller;
pub(crate) mod damage_numbers;
pub(crate) mod enemy;
pub(crate) mod healthbar;
pub(crate) mod level;
pub(crate) mod overlays;
pub(crate) mod player;
pub(crate) mod simple_animation;
pub(crate) mod waves;
pub(crate) mod weapons;

#[derive(Component, Reflect)]
pub(crate) struct Health(pub f32);

#[derive(Component, Reflect, Default)]
pub struct Speed(pub f32);

#[derive(Event, Reflect)]
pub(crate) struct PickUpWeapon {
    pub weapon_type: WeaponType,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        abilities::plugin,
        damage_numbers::plugin,
        enemy::plugin,
        healthbar::plugin,
        level::plugin,
        overlays::plugin,
        player::plugin,
        weapons::plugin,
        waves::plugin,
        simple_animation::plugin,
    ));
}
