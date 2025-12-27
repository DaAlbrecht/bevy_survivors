use bevy::prelude::*;

pub mod attack;
pub mod collision;
pub mod cooldown;
pub mod hit;
pub mod pickup;
pub mod projectile_movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        attack::plugin,
        collision::plugin,
        cooldown::plugin,
        pickup::plugin,
        projectile_movement::plugin,
        hit::plugin,
    ));
}
