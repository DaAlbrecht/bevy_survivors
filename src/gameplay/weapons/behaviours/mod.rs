use bevy::prelude::*;

pub mod chain;
pub mod falling;
pub mod homing;
pub mod melee;
pub mod nova;
pub mod orbiters;
pub mod shot;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        shot::plugin,
        orbiters::plugin,
        chain::plugin,
        nova::plugin,
        homing::plugin,
        falling::plugin,
        melee::plugin,
    ));
}
