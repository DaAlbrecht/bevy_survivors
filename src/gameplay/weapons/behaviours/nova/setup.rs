use crate::gameplay::weapons::prelude::*;
use bevy::prelude::*;

impl ApplySpec for NovaSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::NovaAttack,
            ProjectileCount(self.projectile_count),
            ProjectileSpeed(self.speed),
            super::SpreadPattern(self.spread_pattern.clone()),
        ));
    }
}
