use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::components::{ProjectileCount, WeaponLifetime};

mod attack;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_orbiters_attack);
    app.add_plugins(movement::plugin);
}

#[derive(Component)]
pub struct OrbitersAttack;
#[derive(Component)]
pub struct OrbitRadius(pub f32);
#[derive(Component)]
pub struct OrbitAngularSpeed(pub f32);

#[derive(Component)]
pub struct OrbiterProjectile;

#[derive(Component)]
pub struct OrbitPhase(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrbitersSpec {
    pub count: u32,
    pub radius: f32,
    pub angular_speed: f32,
    pub lifetime: f32,
    pub damage: f32,
}

impl EntityCommand for OrbitersSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((
            OrbitersAttack,
            ProjectileCount(self.count),
            OrbitRadius(self.radius),
            OrbitAngularSpeed(self.angular_speed),
            WeaponLifetime(self.lifetime),
        ));
    }
}
