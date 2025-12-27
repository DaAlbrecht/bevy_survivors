use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_falling_attack);
}

#[derive(Component)]
pub struct FallingAttack;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FallingSpec {
    pub spawn_height: f32,
    pub fall_speed: f32,
    pub explosion_radius: Option<f32>,
}

#[derive(Component)]
pub struct SpawnHeight(pub f32);
