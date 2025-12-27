use bevy::prelude::*;

use crate::gameplay::player::{InInventoryOf, Player};
use crate::gameplay::weapons::prelude::*;

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
    weapon_assets: Res<WeaponMap>,
) -> Result {
    let kind = trigger.kind;
    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    let Some(spec) = weapon_assets.get(&kind) else {
        error!("No WeaponSpec registered for kind {kind:?}");
        return Ok(());
    };

    let w_entity = commands
        .spawn((
            Name::new(format!("{kind:?}")),
            Weapon,
            kind,
            InInventoryOf(player),
            BaseDamage(spec.base_damage),
            WeaponCooldown(Timer::from_seconds(spec.cooldown, TimerMode::Once)),
            WeaponProjectileVisuals(spec.visuals.clone()),
        ))
        .id();

    if spec.despawn_on_hit {
        commands.entity(w_entity).insert(DeathOnCollision);
    }

    match spec.dot {
        Some(dot) => {
            commands.entity(w_entity).insert(TickDuration(dot));
        }
        None => {
            commands.entity(w_entity).insert(CollisionDamage);
        }
    }

    if let Some(impact) = &spec.impact_visuals {
        commands
            .entity(w_entity)
            .insert(WeaponImpactVisuals(impact.clone()));
    }

    spec.attack.apply(&mut commands, w_entity);
    spec.on_hit.apply(&mut commands, w_entity);
    spec.sfx.apply(&mut commands, w_entity);

    Ok(())
}
