use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    GameLayer, PausableSystems,
    gameplay::{
        PickUpWeapon, Speed,
        damage_numbers::DamageType,
        enemy::{DamageCooldown, Enemy, EnemyDamageEvent},
        player::{AddToInventory, Direction, Inventory, Player},
        weapons::{
            circles::{Circles, patch_circles, spawn_circles},
            dot::Bleed,
            energy::{Energy, patch_energy, spawn_energy_projectiles},
            fireball::{Fireball, patch_fireball, spawn_fireball_projectile},
            icelance::{Icelance, patch_icelance, spawn_icelance_projectile},
            lightning::{Lightning, patch_lightning, spawn_lightning_bolt},
            orbs::{Orb, patch_orb, spawn_orb_projectile},
            scale::{Scale, patch_scale, spawn_scale_projectile},
            thorn::{Thorn, patch_thorn, spawn_thorn_projectile},
            weaponstats::{
                make_circles_levels, make_energy_levels, make_fireball_levels,
                make_icelance_levels, make_lightning_levels, make_orb_levels, make_scale_levels,
                make_thorn_levels,
            },
        },
    },
    screens::Screen,
};

pub mod circles;
pub mod dot;
pub mod energy;
pub mod fireball;
pub mod icelance;
pub mod lightning;
pub mod orbs;
pub mod scale;
pub mod thorn;
pub mod weaponstats;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        energy::plugin,
        circles::plugin,
        // scale::plugin,
        // fireball::plugin,
        lightning::plugin,
        // icelance::plugin,
        orbs::plugin,
        thorn::plugin,
        dot::plugin,
    ));

    app.add_systems(
        FixedUpdate,
        (handle_timers, attack, move_projectile)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_observer(add_weapon_to_inventory);

    app.register_type::<WeaponType>();
}

#[derive(Component, Reflect)]
#[require(
    RigidBody::Kinematic,
    Collider= Collider::rectangle(16., 16.) ,
    DebugRender = DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
    CollisionEventsEnabled,
    CollisionLayers =  CollisionLayers::new(
        GameLayer::Player,
        [
            GameLayer::Enemy,
            GameLayer::Default,
        ],
    ),
)]
pub(crate) struct PlayerProjectile;

#[derive(Component, Default, Reflect)]
pub(crate) struct Weapon;

#[derive(Component)]
#[relationship(relationship_target = WeaponProjectiles)]
#[derive(Reflect)]
pub(crate) struct CastWeapon(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CastWeapon, linked_spawn)]
#[derive(Reflect)]
pub(crate) struct WeaponProjectiles(Vec<Entity>);

#[derive(Component, Reflect)]
pub(crate) struct ProjectileCount(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct ExplosionRadius(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Duration(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Knockback(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Root(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Lifetime(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct MaxHitCount(pub usize);

//Generic Components used widely
//****************************************************************
#[derive(Component, Reflect)]
pub(crate) struct Damage(pub f32);

#[derive(Component, Default, Reflect)]
pub(crate) struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Range(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Halt;

#[derive(Component, Reflect)]
pub(crate) struct Despawn;

//****************************************************************

#[derive(Component, Clone, Copy, PartialEq, Debug, Reflect)]
pub(crate) enum WeaponType {
    Energy,
    Circles,
    Scale,
    Icelance,
    Fireball,
    Lightning,
    Orb,
    Thorn,
}

impl WeaponType {
    pub const ALL: [WeaponType; 8] = [
        WeaponType::Energy,
        WeaponType::Circles,
        WeaponType::Scale,
        WeaponType::Fireball,
        WeaponType::Icelance,
        WeaponType::Lightning,
        WeaponType::Orb,
        WeaponType::Thorn,
    ];
}

#[derive(EntityEvent)]
pub(crate) struct WeaponPatchEvent {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub(crate) struct WeaponAttackEvent {
    pub entity: Entity,
}

pub(crate) fn add_weapon_to_inventory(
    trigger: On<PickUpWeapon>,
    mut commands: Commands,
    player_q: Query<Entity, (With<Player>, Without<Weapon>)>,
    owned_weapons: Query<(Entity, &WeaponType), With<Weapon>>,
) -> Result {
    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    // Check if weapon is already owned - if so, upgrade it
    for (weapon_entity, owned_weapon) in &owned_weapons {
        if *owned_weapon == trigger.weapon_type {
            info!("Upgrading Weapon: {:?}", owned_weapon);
            // Trigger upgrade event on the weapon entity itself
            commands.trigger(WeaponPatchEvent {
                entity: weapon_entity,
            });
            return Ok(());
        }
    }

    let mut inventory_entry = commands.spawn(AddToInventory(player));

    match trigger.weapon_type {
        WeaponType::Energy => {
            inventory_entry.insert(Energy);
            inventory_entry.observe(patch_energy);
            inventory_entry.observe(spawn_energy_projectiles);
            let entity = inventory_entry.id();
            commands.insert_resource(make_energy_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Circles => {
            inventory_entry.insert(Circles);
            inventory_entry.observe(patch_circles);
            inventory_entry.observe(spawn_circles);
            let entity = inventory_entry.id();
            commands.insert_resource(make_circles_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Scale => {
            inventory_entry.insert(Scale);
            inventory_entry.observe(patch_scale);
            inventory_entry.observe(spawn_scale_projectile);
            let entity = inventory_entry.id();
            commands.insert_resource(make_scale_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Fireball => {
            inventory_entry.insert(Fireball);
            inventory_entry.observe(patch_fireball);
            inventory_entry.observe(spawn_fireball_projectile);
            let entity = inventory_entry.id();
            commands.insert_resource(make_fireball_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Icelance => {
            inventory_entry.insert(Icelance);
            inventory_entry.observe(patch_icelance);
            inventory_entry.observe(spawn_icelance_projectile);
            let entity = inventory_entry.id();
            commands.insert_resource(make_icelance_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Lightning => {
            inventory_entry.insert(Lightning);
            inventory_entry.observe(patch_lightning);
            inventory_entry.observe(spawn_lightning_bolt);
            let entity = inventory_entry.id();
            commands.insert_resource(make_lightning_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Orb => {
            inventory_entry.insert(Orb);
            inventory_entry.observe(patch_orb);
            inventory_entry.observe(spawn_orb_projectile);
            let entity = inventory_entry.id();
            commands.insert_resource(make_orb_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
        WeaponType::Thorn => {
            inventory_entry.insert(Thorn);
            inventory_entry.observe(patch_thorn);
            inventory_entry.observe(spawn_thorn_projectile);
            let entity = inventory_entry.id();
            commands.insert_resource(make_thorn_levels());
            commands.trigger(WeaponPatchEvent { entity });
        }
    }

    Ok(())
}

fn attack(
    player_q: Query<(Entity, &Transform), With<Player>>,
    inventory: Query<&Inventory>,
    mut weapons: Query<(Entity, &mut Cooldown, Option<&Range>), With<Weapon>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let Ok((player, player_transform)) = player_q.single() else {
        return Ok(());
    };

    for inventory_slot in inventory.iter_descendants(player) {
        let (weapon, mut cooldown, range) = weapons.get_mut(inventory_slot)?;
        let mut in_range = true;
        if let Some(range) = range {
            for enemy_transform in &enemy_q {
                let distance = enemy_transform
                    .translation
                    .truncate()
                    .distance(player_transform.translation.truncate());
                in_range = range.0 >= distance;
            }
        }

        if cooldown.0.is_finished() && in_range {
            commands.trigger(WeaponAttackEvent { entity: weapon });
            cooldown.0.reset();
        }
    }

    Ok(())
}

fn handle_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut Cooldown, With<Weapon>>,
    mut durations: Query<&mut Duration, With<CastWeapon>>,
    mut thorn_dmg_timer: Query<&mut DamageCooldown, With<Thorn>>,
    mut root_timer: Query<(Entity, &mut Root), With<Enemy>>,
    mut bleed_timer: Query<(Entity, &mut Bleed), With<Enemy>>,
    mut commands: Commands,
) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }

    for mut dmg_timer in &mut thorn_dmg_timer {
        dmg_timer.0.tick(time.delta());
    }

    for (entity, mut root) in &mut root_timer {
        root.0.tick(time.delta());
        if root.0.is_finished() {
            commands.entity(entity).remove::<Root>();
        }
    }

    for (target, mut bleed) in &mut bleed_timer {
        bleed.duration.tick(time.delta());
        bleed.tick.tick(time.delta());
        if bleed.tick.is_finished() {
            commands.trigger(EnemyDamageEvent {
                entity_hit: target,
                dmg: bleed.dmg_per_tick,
                damage_type: DamageType::Physical,
            });
            bleed.tick.reset();
        }
        if bleed.duration.is_finished() {
            commands.entity(target).remove::<Bleed>();
        }
    }
}

fn move_projectile(
    weapons: Query<(Entity, &Speed), With<Weapon>>,
    projectiles: Query<&WeaponProjectiles>,
    mut projectile_q: Query<
        (&mut LinearVelocity, &Direction, Option<&Halt>),
        With<PlayerProjectile>,
    >,
) {
    // Loop over all types of weapons
    for (weapon, speed) in &weapons {
        // Iterate over each projectile for this given weapon type

        for projectile in projectiles.iter_descendants(weapon) {
            let Ok((mut linear_velocity, bullet_direction, halt)) =
                projectile_q.get_mut(projectile)
            else {
                continue;
            };

            if halt.is_some() {
                linear_velocity.0.x = 0.0;
                linear_velocity.0.y = 0.0;
                continue;
            }

            let movement = bullet_direction.0.normalize_or_zero() * speed.0;
            linear_velocity.0.x = movement.x;
            linear_velocity.0.y = movement.y;
        }
    }
}
