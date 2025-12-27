use crate::gameplay::weapons::{
    behaviours::WeaponProjectileVisuals,
    components::{CastWeapon, PlayerProjectile, ProjectileDirection},
    systems::attack::WeaponAttackEvent,
};
use bevy::prelude::*;

use crate::gameplay::{enemy::Enemy, player::Player};

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
        // Spawn above
        let spawn_position = Vec3::new(
            enemy_pos.translation.x,
            enemy_pos.translation.y + spawn_height.0,
            10.0,
        );

        let fall_direction = Vec3::new(0.0, -1.0, 0.0);

        let mut proj = commands.spawn((
            Name::new("Falling Projectile"),
            CastWeapon(weapon),
            Transform::from_translation(spawn_position),
            ProjectileDirection(fall_direction),
            PlayerProjectile,
        ));

        projectile_visuals.0.apply_ec(&mut proj);
    }

    Ok(())
}
