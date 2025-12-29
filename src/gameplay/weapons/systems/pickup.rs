use crate::gameplay::weapons::{AddWeapon, components::Weapon, kind::WeaponKind, spec::WeaponMap};
use bevy::prelude::*;

use crate::gameplay::player::{InInventoryOf, Player};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(handle_pickup_weapon);
    app.add_observer(spawn_weapon_instance);
}

#[derive(Event, Reflect, Clone, Copy, Debug)]
pub struct PickUpWeaponEvent {
    pub kind: WeaponKind,
}

#[derive(Event)]
pub struct SpawnWeaponInstanceEvent {
    pub kind: WeaponKind,
}

#[derive(Event)]
#[allow(dead_code)]
pub struct UpgradeWeaponEvent {
    pub kind: WeaponKind,
    pub amount: u32,
}

pub fn handle_pickup_weapon(
    trigger: On<PickUpWeaponEvent>,
    player_q: Query<Entity, With<Player>>,
    weapons_in_inventories: Query<(&WeaponKind, &InInventoryOf), With<Weapon>>,
    mut commands: Commands,
) -> Result {
    let kind = trigger.kind;
    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    let owned = weapons_in_inventories
        .iter()
        .any(|(k, rel)| *k == kind && rel.0 == player);

    if owned {
        commands.trigger(UpgradeWeaponEvent { kind, amount: 1 });
    } else {
        commands.trigger(SpawnWeaponInstanceEvent { kind });
    }

    Ok(())
}

pub fn spawn_weapon_instance(
    trigger: On<SpawnWeaponInstanceEvent>,
    mut commands: Commands,
    weapon_assets: Res<WeaponMap>,
) {
    let kind = trigger.kind;

    let Some(spec) = weapon_assets.get(&kind) else {
        error!("No WeaponSpec registered for kind {kind:?}");
        return;
    };

    commands.queue(AddWeapon(spec.clone()));
}
