use bevy::prelude::*;

//TODO: MOVE THE TICK PART INTO ENEMY ONCE REFACTORED
pub mod collision;
pub mod projectile_movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((collision::plugin, projectile_movement::plugin));
}
