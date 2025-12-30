use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::weapons::{
    behaviours::TriggerAttackBehavior,
    components::{ProjectileCount, WeaponRange},
};

mod attack;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(attack::on_chain_attack);
}

#[derive(Component, Event)]
pub struct ChainAttack;

#[derive(Component)]
pub struct ChainLifetime(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainSpec {
    pub max_hits: u32,
    pub range: f32,
    pub bolt_lifetime: f32,
}

impl EntityCommand for ChainSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((
            ChainAttack,
            ProjectileCount(self.max_hits),
            WeaponRange(self.range),
            ChainLifetime(self.bolt_lifetime),
        ));
    }
}

impl TriggerAttackBehavior for ChainSpec {
    fn trigger(&self, mut commands: Commands) {
        commands.trigger(ChainAttack);
    }
}
