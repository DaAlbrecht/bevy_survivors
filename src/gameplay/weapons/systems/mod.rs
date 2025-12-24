use bevy::prelude::*;

pub mod attack;
pub mod cooldown;
pub mod hit;
pub mod pickup;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        attack::plugin,
        cooldown::plugin,
        pickup::plugin,
        hit::plugin,
    ));
}
