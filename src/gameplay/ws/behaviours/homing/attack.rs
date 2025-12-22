use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    GameLayer,
    gameplay::{
        enemy::Enemy,
        player::Player,
        ws::{prelude::*, systems::cooldown::WeaponDuration},
    },
};

pub fn on_homing_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<
        (
            &ProjectileCount,
            &WeaponLifetime,
            &super::MaxHits,
            &super::MovementConfig,
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
        // Pick initial target: cycle through enemies if multiple, use same if only one
        let initial_target = if enemy_count > 0 {
            // For single enemy, add delay between projectiles. For multiple, distribute evenly
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
            super::HomingProjectile,
            super::CurrentTarget(initial_target),
            super::HitCounter {
                hits: 0,
                max_hits: max_hits.0 as usize,
            },
            movement_config.clone(),
            WeaponDuration(Timer::from_seconds(lifetime.0, TimerMode::Once)),
        ));

        projectile_visuals.0.apply_ec(&mut proj);

        proj.observe(on_homing_hit);
    }

    Ok(())
}

fn on_homing_hit(
    event: On<avian2d::prelude::CollisionStart>,
    mut hit_counter_q: Query<&mut super::HitCounter>,
    weapon_stats_q: Query<(&HitSpec, &BaseDamage)>,
    cast_q: Query<&CastWeapon>,
    enemy_q: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;
    let target = event.collider2;

    let weapon = cast_q.get(projectile)?.0;
    let enemy_tf = enemy_q.get(target)?;

    let (hit, dmg) = weapon_stats_q.get(weapon)?;

    commands.trigger(WeaponHitEvent {
        entity: weapon,
        target,
        hit_pos: enemy_tf.translation,
        dmg: dmg.0,
        damage_type: hit.damage_type,
        aoe: None,
        effects: hit.effects.clone(),
    });

    if let Ok(mut counter) = hit_counter_q.get_mut(projectile) {
        counter.hits += 1;

        if counter.hits >= counter.max_hits {
            commands.entity(projectile).despawn();
        }
    }

    Ok(())
}
