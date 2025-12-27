use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod movement;
mod setup;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_zone_attack);
    app.add_plugins(movement::plugin);
}

#[derive(Component)]
pub struct ZoneAttack;

#[derive(Component)]
pub struct ZoneAttackInstance;

#[derive(Component)]
pub struct ZoneFollowPlayer;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ZoneTarget {
    Player,
    Enemy,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ZoneShape {
    Circle { radius: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneSpec {
    pub shape: ZoneShape,
    pub target: ZoneTarget,
    pub lifetime: f32,
}
