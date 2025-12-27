use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod movement;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_homing_attack);
    app.add_plugins(movement::plugin);
}

#[derive(Component)]
pub struct HomingAttack;

#[derive(Component, Reflect)]
pub struct HomingProjectile;

#[derive(Component, Reflect)]
pub struct CurrentTarget(pub Option<Entity>);

#[derive(Component, Reflect)]
pub struct HitCounter {
    pub hits: usize,
    pub max_hits: usize,
}

#[derive(Component, Reflect)]
pub struct MaxHits(pub u32);

#[derive(Component, Clone, Reflect)]
pub struct MovementConfig {
    pub pattern: MovementPatternKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomingSpec {
    pub count: u32,
    pub lifetime: f32,
    pub max_hits: u32,
    pub movement: MovementPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementPattern {
    pub kind: MovementPatternKind,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub enum MovementPatternKind {
    Straight,
    Zigzag { frequency: f32, amplitude: f32 },
    Wave { frequency: f32, amplitude: f32 },
    Spiral { rotation_speed: f32 },
}
