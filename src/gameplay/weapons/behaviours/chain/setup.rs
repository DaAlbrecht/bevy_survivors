use bevy::prelude::*;

use crate::gameplay::weapons::{
    ApplySpec,
    behaviours::chain::ChainSpec,
    components::{ProjectileCount, WeaponRange},
};

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
