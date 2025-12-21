use crate::gameplay::ws::{
    behaviours::shared::{ExplosionRadius, ProjectileSpeed},
    prelude::*,
};
use bevy::prelude::*;

impl ApplySpec for ShotSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((super::ShotAttack, ProjectileSpeed(self.speed)));

        if let Some(r) = self.explosion_radius {
            ec.insert(ExplosionRadius(r));
        }
    }
}
