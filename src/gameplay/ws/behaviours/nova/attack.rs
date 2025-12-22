use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::gameplay::{
    player::Player,
    ws::{prelude::*, runtime::sfx::WeaponAttackSfx},
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

        proj.observe(on_nova_hit);
    }

    Ok(())
}

fn on_nova_hit(
    event: On<avian2d::prelude::CollisionStart>,
    weapon_stats_q: Query<(&HitSpec, &BaseDamage, Option<&ExplosionRadius>)>,
    cast_q: Query<&CastWeapon>,
    enemy_q: Query<&Transform>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;
    let target = event.collider2;

    let weapon = cast_q.get(projectile)?.0;
    let enemy_tf = enemy_q.get(target)?;

    let (hit, dmg, explosion_radius) = weapon_stats_q.get(weapon)?;

    commands.trigger(WeaponHitEvent {
        entity: weapon,
        target,
        hit_pos: enemy_tf.translation,
        dmg: dmg.0,
        damage_type: hit.damage_type,
        aoe: explosion_radius.map(|er| er.0),
        effects: hit.effects.clone(),
    });

    commands.entity(projectile).despawn();
    Ok(())
}
