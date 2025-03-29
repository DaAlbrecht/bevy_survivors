use bevy::prelude::*;

use crate::player::SpawnPlayer;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_level);
}

pub fn spawn_level(world: &mut World) {
    SpawnPlayer { max_speed: 300.0 }.apply(world);
}
