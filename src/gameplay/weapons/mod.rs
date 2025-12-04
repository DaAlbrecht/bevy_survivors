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
            circles::{Circles, CirclesAttackEvent, upgrade_circles},
            dot::Bleed,
            energy::{Energy, EnergyAttackEvent, upgrade_energy},
            fireball::{Fireball, FireballAttackEvent, upgrade_fireball},
            icelance::{Icelance, upgrade_icelance},
            lightning::{Lightning, LightningAttackEvent, upgrade_lightning},
            orbs::{Orb, OrbAttackEvent, upgrade_orb},
            scale::{Scale, ScaleAttackEvent, upgrade_scale},
            thorn::{Thorn, ThornAttackEvent, upgrade_thorn},
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

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        energy::plugin,
        circles::plugin,
        scale::plugin,
        fireball::plugin,
        lightning::plugin,
        icelance::plugin,
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
pub(crate) struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Knockback(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Damage(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Range(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct ExplosionRadius(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct WeaponDuration(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct ProjectileCount(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Halt;

#[derive(Component, Reflect)]
pub(crate) struct StartPosition(Vec2);

#[derive(Component, Reflect)]
pub(crate) struct Despawn;

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

#[derive(Component, Default, Reflect)]
pub(crate) struct Segmented;

#[derive(Component, Reflect)]
pub(crate) struct Root(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Tail;

#[derive(Component)]
pub struct WeaponTick(pub Timer);

#[derive(EntityEvent)]
pub(crate) struct UpgradeWeaponEvent {
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
    //
    // Check if weapon is already owned - if so, upgrade it
    for (weapon_entity, owned_weapon) in &owned_weapons {
        if *owned_weapon == trigger.weapon_type {
            info!("Upgrading Weapon: {:?}", owned_weapon);
            // Trigger upgrade event on the weapon entity itself
            commands.trigger(UpgradeWeaponEvent {
                entity: weapon_entity,
            });
            return Ok(());
        }
    }

    //Get Inventory of Player
    let mut e = commands.spawn(AddToInventory(player));

    match trigger.weapon_type {
        WeaponType::Energy => {
            e.insert(Energy);
            e.observe(upgrade_energy);
        }
        WeaponType::Circles => {
            e.insert(Circles);
            e.observe(upgrade_circles);
        }
        WeaponType::Scale => {
            e.insert(Scale);
            e.observe(upgrade_scale);
        }
        WeaponType::Fireball => {
            e.insert(Fireball);
            e.observe(upgrade_fireball);
        }
        WeaponType::Icelance => {
            e.insert(Icelance);
            e.observe(upgrade_icelance);
        }
        WeaponType::Lightning => {
            e.insert(Lightning);
            e.observe(upgrade_lightning);
        }
        WeaponType::Orb => {
            e.insert(Orb);
            e.observe(upgrade_orb);
        }
        WeaponType::Thorn => {
            e.insert(Thorn);
            e.observe(upgrade_thorn);
        }
    }

    Ok(())
}

fn attack(
    player_q: Query<Entity, With<Player>>,
    inventory: Query<&Inventory>,
    mut weapons: Query<(&mut Cooldown, &WeaponType), With<Weapon>>,
    mut commands: Commands,
) -> Result {
    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    for inventory_slot in inventory.iter_descendants(player) {
        let (mut cooldown, weapon_type) = weapons.get_mut(inventory_slot)?;

        if cooldown.0.is_finished() {
            match weapon_type {
                WeaponType::Energy => commands.trigger(EnergyAttackEvent),
                WeaponType::Circles => commands.trigger(CirclesAttackEvent),
                WeaponType::Scale => commands.trigger(ScaleAttackEvent),
                WeaponType::Fireball => commands.trigger(FireballAttackEvent),
                WeaponType::Icelance => commands.trigger(icelance::IcelanceAttackEvent),
                WeaponType::Lightning => commands.trigger(LightningAttackEvent),
                WeaponType::Orb => commands.trigger(OrbAttackEvent),
                WeaponType::Thorn => commands.trigger(ThornAttackEvent),
            }
            cooldown.0.reset();
        }
    }

    Ok(())
}

fn handle_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut Cooldown, With<Weapon>>,
    mut durations: Query<&mut WeaponDuration, With<CastWeapon>>,
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
