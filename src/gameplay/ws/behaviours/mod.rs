use bevy::prelude::*;

pub mod chain;
pub mod orbiters;
pub mod shared;
pub mod shot;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(shot::plugin);
}
