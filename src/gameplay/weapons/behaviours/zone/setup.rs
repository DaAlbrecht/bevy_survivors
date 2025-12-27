use bevy::prelude::*;

use crate::gameplay::weapons::prelude::{ApplySpec, WeaponLifetime};

impl ApplySpec for super::ZoneSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            super::ZoneAttack,
            self.target.clone(),
            self.shape.clone(),
            WeaponLifetime(self.lifetime),
        ));
    }
}
