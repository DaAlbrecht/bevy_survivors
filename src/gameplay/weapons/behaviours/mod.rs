use bevy::prelude::*;

pub mod chain;
pub mod falling;
pub mod homing;
pub mod nova;
pub mod orbiters;
pub mod shared;
pub mod shot;
pub mod zone;

pub(super) fn plugin(app: &mut App) {
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
