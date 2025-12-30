use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::{
    behaviours::TriggerAttackBehavior,
    components::{ExplosionRadius, ProjectileSpeed},
};

mod attack;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_projectile_attack);
}

#[derive(Component, Event)]
pub struct ShotAttack;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotSpec {
    pub speed: f32,
    pub range: f32,
    pub explosion_radius: Option<f32>,
}

impl EntityCommand for ShotSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((ShotAttack, ProjectileSpeed(self.speed)));

        if let Some(radius) = self.explosion_radius {
            entity.insert(ExplosionRadius(radius));
        }
    }
}

impl TriggerAttackBehavior for ShotSpec {
    fn trigger(&self, mut commands: Commands) {
        commands.trigger(ShotAttack);
    }
}
