use bevy::prelude::*;

use crate::gameplay::weapons::{
    ApplySpec,
    behaviours::homing::{HomingSpec, MaxHits, MovementConfig},
    components::{ProjectileCount, ProjectileSpeed, WeaponLifetime},
};

impl ApplySpec for HomingSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::HomingAttack,
            ProjectileCount(self.count),
            ProjectileSpeed(self.movement.speed),
            WeaponLifetime(self.lifetime),
            MaxHits(self.max_hits),
            MovementConfig {
                pattern: self.movement.kind.clone(),
            },
        ));
    }
}
