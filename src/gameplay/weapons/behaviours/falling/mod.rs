use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::{
    ApplySpec,
    components::{ExplosionRadius, ProjectileSpeed},
};

mod attack;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_falling_attack);
}

#[derive(Component)]
pub struct FallingAttack;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FallingSpec {
    pub spawn_height: f32,
    pub fall_speed: f32,
    pub explosion_radius: Option<f32>,
}

impl ApplySpec for FallingSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            FallingAttack,
            SpawnHeight(self.spawn_height),
            ProjectileSpeed(self.fall_speed),
        ));

        if let Some(radius) = self.explosion_radius {
            ec.insert(ExplosionRadius(radius));
        }
    }
}

#[derive(Component)]
pub struct SpawnHeight(pub f32);
