use bevy::prelude::*;

use crate::gameplay::weapons::{prelude::*, runtime::ApplySpec};

use super::{ConeConfig, ZoneAttack, ZoneSpec};

impl ApplySpec for ZoneSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            ZoneAttack,
            WeaponLifetime(self.duration),
            ZoneWidth(self.width),
        ));

        if self.follow_player {
            entity_commands.insert(FollowPlayer);
        }

        if let Some(cone_config) = &self.cone {
            entity_commands.insert(ZoneConeConfig(cone_config.clone()));
        }
    }
}

#[derive(Component, Reflect, Clone)]
pub struct ZoneWidth(pub f32);

#[derive(Component, Reflect)]
pub struct FollowPlayer;

#[derive(Component, Reflect, Clone)]
pub struct ZoneConeConfig(pub ConeConfig);
