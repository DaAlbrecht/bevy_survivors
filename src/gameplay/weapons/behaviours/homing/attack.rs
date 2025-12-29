use crate::{
    GameLayer,
    gameplay::{
        enemy::Enemy,
        player::Player,
        weapons::{
            behaviours::{
                WeaponProjectileVisuals,
                homing::{CurrentTarget, HitCounter, HomingProjectile, MaxHits, MovementConfig},
            },
            components::{CastWeapon, ProjectileCount, ProjectileDirection, WeaponLifetime},
            systems::{attack::WeaponAttackEvent, cooldown::WeaponDuration},
        },
    },
};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

pub fn on_homing_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<
        (
            &ProjectileCount,
            &WeaponLifetime,
            &MaxHits,
            &MovementConfig,
            &WeaponProjectileVisuals,
        ),
        With<super::HomingAttack>,
    >,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((count, lifetime, max_hits, movement_config, projectile_visuals)) = weapon_q.get(weapon)
    else {
        return Ok(());
    };

    let player_pos = player_q.single()?;

    let enemy_count = enemy_q.iter().len();
    let mut rng = rand::rng();

    for i in 0..count.0 {
        let initial_target = if enemy_count > 0 {
            let target_idx = if enemy_count == 1 {
                0
            } else {
                (i as usize + rng.random_range(0..enemy_count)) % enemy_count
            };
            enemy_q.iter().nth(target_idx)
        } else {
            None
        };

        let mut proj = commands.spawn((
            Name::new("Homing Projectile"),
            CastWeapon(weapon),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.0),
            ProjectileDirection(Vec3::ZERO), // Will be updated by movement system
            RigidBody::Kinematic,
            Collider::rectangle(32.0, 32.0),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
            LinearVelocity(Vec2::ZERO),
            HomingProjectile,
            CurrentTarget(initial_target),
            HitCounter {
                hits: 0,
                max_hits: max_hits.0 as usize,
            },
            movement_config.clone(),
            WeaponDuration(Timer::from_seconds(lifetime.0, TimerMode::Once)),
        ));

        projectile_visuals.0.apply_ec(&mut proj);

        proj.observe(on_homing_hit_counter);
    }

    Ok(())
}

// Separate observer to track hit count for homing projectiles
fn on_homing_hit_counter(
    event: On<avian2d::prelude::CollisionStart>,
    mut hit_counter_q: Query<&mut super::HitCounter>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;

    if let Ok(mut counter) = hit_counter_q.get_mut(projectile) {
        counter.hits += 1;

        if counter.hits >= counter.max_hits {
            commands.entity(projectile).despawn();
        }
    }

    Ok(())
}
