use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_nova_attack);
}

#[derive(Component)]
pub struct NovaAttack;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NovaSpec {
    pub speed: f32,
    pub projectile_count: u32,
    pub spread_pattern: SpreadPatternKind,
}

#[derive(Component, Debug, Clone)]
pub struct SpreadPattern(pub SpreadPatternKind);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpreadPatternKind {
    Even,
    Random,
}
