use bevy::prelude::*;

use crate::gameplay::{enemy::Enemy, player::Player, ws::prelude::*};

pub fn on_falling_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<(&super::SpawnHeight, &WeaponProjectileVisuals), With<super::FallingAttack>>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((spawn_height, projectile_visuals)) = weapon_q.get(weapon) else {
        return Ok(());
    };

    let player_pos = player_q.single()?;

    // Find closest enemy to player
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
        // Spawn projectile ABOVE the enemy position
        let spawn_position = Vec3::new(
            enemy_pos.translation.x,
            enemy_pos.translation.y + spawn_height.0,
            10.0,
        );

        // Falls straight down
        let fall_direction = Vec3::new(0.0, -1.0, 0.0);

        let mut proj = commands.spawn((
            Name::new("Falling Projectile"),
            CastWeapon(weapon),
            Transform::from_xyz(spawn_position.x, spawn_position.y, 10.0),
            ProjectileDirection(fall_direction),
            PlayerProjectile,
        ));

        projectile_visuals.0.apply_ec(&mut proj);

        proj.observe(on_falling_hit);
    }

    Ok(())
}

fn on_falling_hit(
    event: On<avian2d::prelude::CollisionStart>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    cast_q: Query<&CastWeapon>,
    weapon_hit_q: Query<&HitSpec>,
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
        target,
        hit_pos: enemy_tf.0.translation,

        dmg: dmg.0,
        damage_type: hit.damage_type,
        aoe: explosion_radius.map(|er| er.0),

        effects: hit.effects.clone(),
    });

    commands.entity(projectile).despawn();
    Ok(())
}
