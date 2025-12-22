use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;

impl ApplySpec for ChainLightningSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::ChainLightningAttack,
            ProjectileCount(self.max_hits),
            WeaponRange(self.range),
            super::LightningBoltLifetime(self.bolt_lifetime),
        ));
    }
}
