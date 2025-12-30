use crate::gameplay::weapons::behaviours::TriggerAttackBehavior;
use bevy::prelude::*;

use crate::gameplay::weapons::{components::Weapon, kind::WeaponKind, spec::WeaponMap};

/// A player has attacked with a Weapon
///
/// The Entity that this Event is triggered for is the Weapon that
/// attacked.
#[derive(EntityEvent)]
pub struct WeaponAttack {
    pub entity: Entity,
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(dispatch_weapon_attacks);
}

fn dispatch_weapon_attacks(
    attack: On<WeaponAttack>,
    commands: Commands,
    weapon_kind: Query<&WeaponKind, With<Weapon>>,
    weapon_map: Res<WeaponMap>,
) {
    if let Ok(Some(spec)) = weapon_kind
        .get(attack.entity)
        .map(|kind| weapon_map.get(kind))
    {
        spec.attack.trigger(commands);
    }
}
