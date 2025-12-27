use crate::gameplay::{enemy::Enemy, player::Player, weapons::prelude::*};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn on_projectile_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<&WeaponProjectileVisuals, With<super::ShotAttack>>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok(projectile_visuals) = weapon_q.get(weapon) else {
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
            Mass(0.1),
            PlayerProjectile,
        ));

        projectile_visuals.0.apply_ec(&mut proj);
    }

    Ok(())
}
