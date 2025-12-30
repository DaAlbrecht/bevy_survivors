use crate::{
    GameLayer,
    gameplay::{
        player::Player,
        weapons::{
            behaviours::{
                WeaponProjectileVisuals,
                orbiters::{
                    OrbitAngularSpeed, OrbitPhase, OrbitRadius, OrbiterProjectile, OrbitersAttack,
                },
            },
            components::{CastWeapon, PlayerProjectile, ProjectileCount, WeaponLifetime},
            systems::cooldown::WeaponDuration,
        },
    },
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn on_orbiters_attack(
    _orbiters_attack: On<OrbitersAttack>,
    weapon: Single<
        (
            Entity,
            &ProjectileCount,
            &OrbitRadius,
            &OrbitAngularSpeed,
            &WeaponLifetime,
            &WeaponProjectileVisuals,
        ),
        With<OrbitersAttack>,
    >,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let (entity, count, radius, ang_speed, lifetime, projectile_visuals) = weapon.into_inner();

    let player_tf = player_q.single()?;

    let count_f = count.0.max(1) as f32;

    for i in 0..(count.0.max(1) as usize) {
        let phase = std::f32::consts::TAU * (i as f32 / count_f);
        let offset = Vec2::from_angle(phase) * radius.0;
        let world_pos = player_tf.translation + offset.extend(10.0);

        let mut e = commands.spawn((
            Name::new("Orbiter"),
            CastWeapon(entity),
            PlayerProjectile,
            OrbiterProjectile,
            OrbitPhase(phase),
            OrbitRadius(radius.0),
            OrbitAngularSpeed(ang_speed.0),
            WeaponDuration(Timer::from_seconds(lifetime.0, TimerMode::Once)),
            Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
            // physics
            Collider::rectangle(16., 16.),
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
        ));

        projectile_visuals.0.apply_ec(&mut e);
    }

    Ok(())
}
