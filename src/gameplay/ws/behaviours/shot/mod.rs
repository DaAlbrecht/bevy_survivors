use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod movement;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_projectile_attack);
    app.add_plugins(movement::plugin);
}

#[derive(Component)]
pub struct ShotAttack;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotSpec {
    pub speed: f32,
    pub range: f32,
    pub explosion_radius: Option<f32>,
}
