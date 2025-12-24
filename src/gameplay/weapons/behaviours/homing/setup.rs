use crate::gameplay::weapons::prelude::*;
use bevy::prelude::*;

impl ApplySpec for HomingSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::HomingAttack,
            ProjectileCount(self.count),
            ProjectileSpeed(self.movement.speed),
            WeaponLifetime(self.lifetime),
            super::MaxHits(self.max_hits),
            super::MovementConfig {
                pattern: self.movement.kind.clone(),
            },
        ));
    }
}
