use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod attack;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_melee_attack);
    app.add_plugins(movement::plugin);
}

#[derive(Component)]
pub struct MeleeAttack;

#[derive(Component)]
pub struct MeleeAttackZone;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackCone {
    pub angle: f32,
    pub range: f32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeleeSpec {
    /// Whether the zone follows the player
    pub cone: AttackCone,
}

impl EntityCommand for MeleeSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((MeleeAttack, self.cone));
    }
}
