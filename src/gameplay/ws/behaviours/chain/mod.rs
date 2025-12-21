use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod setup;

#[derive(Component)]
pub struct ChainLightningAttack;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainLightningSpec {
    pub max_hits: u32,
    pub range: f32,
    pub damage: f32,
    pub bolt_lifetime: f32,
    pub bolt_thickness: f32,
}

#[derive(Component)]
pub struct LightningBoltLifetime(pub f32);
