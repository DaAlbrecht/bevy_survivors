use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::{
    ApplySpec,
    components::{ExplosionRadius, ProjectileSpeed},
};

mod attack;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_projectile_attack);
}

#[derive(Component)]
pub struct ShotAttack;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotSpec {
    pub speed: f32,
    pub range: f32,
    pub explosion_radius: Option<f32>,
}

impl ApplySpec for ShotSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((ShotAttack, ProjectileSpeed(self.speed)));

        if let Some(r) = self.explosion_radius {
            ec.insert(ExplosionRadius(r));
        }
    }
}
