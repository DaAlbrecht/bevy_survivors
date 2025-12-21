use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;

impl ApplySpec for OrbitersSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert(super::OrbitersAttack);
    }
}
