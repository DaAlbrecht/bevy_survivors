use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;

impl ApplySpec for AttackSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        match self {
            AttackSpec::Orbiters(s) => s.apply(commands, entity),
            AttackSpec::ChainLightning(s) => s.apply(commands, entity),
            AttackSpec::Shot(s) => s.apply(commands, entity),
            AttackSpec::Nova(s) => s.apply(commands, entity),
            AttackSpec::Homing(s) => s.apply(commands, entity),
            AttackSpec::Falling(s) => s.apply(commands, entity),
        }
    }
}

impl ApplySpec for HitSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).insert(self.clone());
    }
}
