use bevy::prelude::*;

pub mod projectile_movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(projectile_movement::plugin);
}
