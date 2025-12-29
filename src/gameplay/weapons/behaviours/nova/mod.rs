use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::components::{ProjectileCount, ProjectileSpeed};

mod attack;

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

impl EntityCommand for NovaSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((
            NovaAttack,
            ProjectileCount(self.projectile_count),
            ProjectileSpeed(self.speed),
            SpreadPattern(self.spread_pattern),
        ));
    }
}

#[derive(Component, Debug, Clone)]
pub struct SpreadPattern(pub SpreadPatternKind);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpreadPatternKind {
    Even,
    Random,
}
