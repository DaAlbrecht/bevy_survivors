use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::{
    behaviours::TriggerAttackBehavior,
    components::{ExplosionRadius, ProjectileSpeed},
};

mod attack;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_falling_attack);
}

#[derive(Component, Event)]
pub struct FallingAttack;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FallingSpec {
    pub spawn_height: f32,
    pub fall_speed: f32,
    pub explosion_radius: Option<f32>,
}

impl EntityCommand for FallingSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((
            FallingAttack,
            SpawnHeight(self.spawn_height),
            ProjectileSpeed(self.fall_speed),
        ));

        if let Some(radius) = self.explosion_radius {
            entity.insert(ExplosionRadius(radius));
        }
    }
}

impl TriggerAttackBehavior for FallingSpec {
    fn trigger(&self, mut commands: Commands) {
        commands.trigger(FallingAttack);
    }
}

#[derive(Component)]
pub struct SpawnHeight(pub f32);
