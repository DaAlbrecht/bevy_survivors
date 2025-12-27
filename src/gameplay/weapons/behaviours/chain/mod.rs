use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_chain_attack);
}

#[derive(Component)]
pub struct ChainAttack;

#[derive(Component)]
pub struct ChainLifetime(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainSpec {
    pub max_hits: u32,
    pub range: f32,
    pub bolt_lifetime: f32,
}
