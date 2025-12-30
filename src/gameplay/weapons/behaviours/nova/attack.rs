use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::gameplay::{
    player::Player,
    weapons::{
        behaviours::{
            WeaponAttackSfx, WeaponProjectileVisuals,
            nova::{NovaAttack, SpreadPattern},
        },
        components::{
            CastWeapon, PlayerProjectile, ProjectileCount, ProjectileDirection, ProjectileSpeed,
        },
    },
};

pub fn on_nova_attack(
    _nova_attack: On<NovaAttack>,
    weapon: Single<
        (
            Entity,
            &ProjectileCount,
            &ProjectileSpeed,
            &SpreadPattern,
            &WeaponProjectileVisuals,
            Option<&WeaponAttackSfx>,
        ),
        With<NovaAttack>,
    >,
    player_pos: Single<&Transform, With<Player>>,
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let (entity, count, _speed, spread_pattern, projectile_visuals, _sfx) = weapon.into_inner();

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
            CastWeapon(entity),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.0)
                .with_rotation(rotation),
            ProjectileDirection(direction.extend(0.0)),
            PlayerProjectile,
        ));

        projectile_visuals.0.apply_ec(&mut proj);
    }

    Ok(())
}
