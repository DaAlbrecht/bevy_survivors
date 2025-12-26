use bevy::prelude::*;

use crate::gameplay::weapons::runtime::ApplySpec;

use super::*;

impl ApplySpec for MeleeSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((MeleeAttack, self.cone.clone()));
    }
}
