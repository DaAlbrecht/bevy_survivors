use avian2d::prelude::*;
use bevy::prelude::*;

use crate::gameplay::{
    enemy::Enemy,
    player::Player,
    ws::{
        behaviours::shared::{
            BaseDamage, CastWeapon, ExplosionRadius, PlayerProjectile, ProjectileSpeed,
        },
        prelude::ProjectileDirection,
        runtime::{attacks::WeaponHit, visuals::WeaponProjectileVisuals},
        systems::{attack::WeaponAttackEvent, hit::WeaponHitEvent},
    },
};

pub fn on_projectile_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<(&ProjectileSpeed, &WeaponProjectileVisuals), With<super::ShotAttack>>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((speed, projectile_visuals)) = weapon_q.get(weapon) else {
        return Ok(());
    };

    let player_pos = player_q.single()?;

    let mut min_distance = f32::MAX;
    let mut closest_enemy: Option<&Transform> = None;

    for enemy_pos in &enemy_q {
        let distance = player_pos
            .translation
            .truncate()
            .distance(enemy_pos.translation.truncate());

        if distance < min_distance {
            min_distance = distance;
            closest_enemy = Some(enemy_pos);
        }
    }

    if let Some(enemy_pos) = closest_enemy {
        let direction = (enemy_pos.translation - player_pos.translation)
            .truncate()
            .normalize();

        let towards_quaternion = Quat::from_rotation_arc(Vec3::Y, direction.extend(0.).normalize());

        let mut proj = commands.spawn((
            Name::new("Projectile"),
            CastWeapon(weapon),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.0)
                .with_rotation(towards_quaternion),
            ProjectileDirection(direction.extend(0.)),
            PlayerProjectile,
        ));

        projectile_visuals.0.apply_ec(&mut proj);

        proj.observe(on_shot_hit);
    }

    Ok(())
}

fn on_shot_hit(
    event: On<CollisionStart>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    cast_q: Query<&CastWeapon>,
    weapon_hit_q: Query<&WeaponHit>,
    weapon_stats_q: Query<(&BaseDamage, Option<&ExplosionRadius>)>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;
    let target = event.collider2;

    let weapon = cast_q.get(projectile)?.0;

    let Ok(enemy_tf) = enemy_q.get(target) else {
        commands.entity(projectile).despawn();
        return Ok(());
    };

    let hit = weapon_hit_q.get(weapon)?;
    let (dmg, explosion_radius) = weapon_stats_q.get(weapon)?;

    commands.trigger(WeaponHitEvent {
        entity: weapon,
        projectile,
        target,
        hit_pos: enemy_tf.0.translation,

        dmg: dmg.0,
        damage_type: hit.0.damage_type,
        aoe: explosion_radius.map(|er| er.0),

        effects: hit.0.effects.clone(),
    });

    commands.entity(projectile).despawn();
    Ok(())
}
