use bevy::prelude::*;

use crate::gameplay::weapons::{
    ApplySpec,
    prelude::{ExplosionRadius, FallingSpec, ProjectileSpeed},
};

impl ApplySpec for FallingSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::FallingAttack,
            super::SpawnHeight(self.spawn_height),
            ProjectileSpeed(self.fall_speed),
        ));

        if let Some(radius) = self.explosion_radius {
            ec.insert(ExplosionRadius(radius));
        }
    }
}
