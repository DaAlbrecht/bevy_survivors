use bevy::prelude::*;

use crate::gameplay::player::{InInventoryOf, Player};
use crate::gameplay::ws::prelude::*;

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
    player_q: Query<Entity, With<Player>>,
    weapon_assets: Res<WeaponAssets>,
    weapon_specs: Res<Assets<WeaponSpec>>,
) -> Result {
    let kind = trigger.kind;
    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    let Some(spec_handle) = weapon_assets.spec_handle_for_kind(kind) else {
        error!("No WeaponSpec handle registered for kind {:?}", kind);
        return Ok(());
    };

    let Some(spec) = weapon_specs.get(spec_handle) else {
        error!(
            "WeaponSpec asset not available for kind {:?} (handle={:?})",
            kind,
            spec_handle.id()
        );
        return Ok(());
    };

    let weapon_e = commands
        .spawn((
            Name::new(spec.name.clone()),
            Weapon,
            kind,
            InInventoryOf(player),
        ))
        .id();

    spec.apply(&mut commands, weapon_e);

    Ok(())
}
