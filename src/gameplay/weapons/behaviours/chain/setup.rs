use crate::gameplay::weapons::prelude::*;
use bevy::prelude::*;

impl ApplySpec for ChainSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::ChainAttack,
            ProjectileCount(self.max_hits),
            WeaponRange(self.range),
            super::ChainLifetime(self.bolt_lifetime),
        ));
    }
}
