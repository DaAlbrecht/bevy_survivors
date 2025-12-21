use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod setup;

#[derive(Component)]
pub struct OrbitersAttack;
#[derive(Component)]
pub struct OrbitRadius(pub f32);
#[derive(Component)]
pub struct OrbitAngularSpeed(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrbitersSpec {
    pub count: u32,
    pub radius: f32,
    pub angular_speed: f32,
    pub lifetime: f32,
    pub damage: f32,
}
