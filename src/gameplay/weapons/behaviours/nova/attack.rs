use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::gameplay::{
    player::Player,
    weapons::{prelude::*, runtime::sfx::WeaponAttackSfx},
};

pub fn on_nova_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<
        (
            &ProjectileCount,
            &ProjectileSpeed,
            &super::SpreadPattern,
            &WeaponProjectileVisuals,
            Option<&WeaponAttackSfx>,
        ),
        With<super::NovaAttack>,
    >,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((count, _speed, spread_pattern, projectile_visuals, _sfx)) = weapon_q.get(weapon) else {
        return Ok(());
    };

    let player_pos = player_q.single()?;

    let num_projectiles = count.0.max(1);
    let angle_step = std::f32::consts::TAU / num_projectiles as f32;

    for i in 0..num_projectiles {
        let angle = match spread_pattern.0 {
            super::SpreadPatternKind::Even => angle_step * i as f32,
            super::SpreadPatternKind::Random => rng.random_range(0.0..std::f32::consts::TAU),
        };

        let direction = Vec2::new(angle.cos(), angle.sin());
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction.extend(0.0).normalize());

        let mut proj = commands.spawn((
            Name::new("Nova Projectile"),
            CastWeapon(weapon),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.0)
                .with_rotation(rotation),
            ProjectileDirection(direction.extend(0.0)),
            PlayerProjectile,
        ));

        projectile_visuals.0.apply_ec(&mut proj);
    }

    Ok(())
}
