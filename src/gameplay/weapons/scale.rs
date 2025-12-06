use avian2d::prelude::CollisionStart;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::damage_numbers::DamageType;
use crate::gameplay::player::Level;
use crate::gameplay::weapons::weaponstats::ScaleLevels;
use crate::gameplay::weapons::{WeaponAttackEvent, WeaponPatchEvent};
use crate::gameplay::{
    Speed,
    enemy::{EnemyDamageEvent, EnemyKnockbackEvent},
    player::{Direction, Player},
    weapons::{CastWeapon, Damage, Knockback, PlayerProjectile, Weapon, WeaponType},
};

use super::Cooldown;

use bevy_rand::{global::GlobalRng, prelude::WyRand};

#[derive(Component)]
#[require(Weapon, WeaponType::Scale, Name::new("Scale"))]
#[derive(Reflect)]
pub(crate) struct Scale;

#[derive(Event, Reflect)]
pub(crate) struct ScaleAttackEvent;

// pub(crate) fn plugin(app: &mut App) {}

pub fn patch_scale(
    _trigger: On<WeaponPatchEvent>,
    mut commands: Commands,
    weapon_q: Query<Entity, With<Scale>>,
    mut weapon_levels: ResMut<ScaleLevels>,
) -> Result {
    let weapon = weapon_q.single()?;

    let Some(stats) = weapon_levels.levels.pop_front() else {
        return Ok(());
    };

    commands
        .entity(weapon)
        .insert(Level(stats.level))
        .insert(Damage(stats.damage))
        .insert(Speed(stats.speed))
        .insert(Knockback(stats.knockback))
        .insert(Cooldown(Timer::from_seconds(
            stats.cooldown,
            TimerMode::Once,
        )));

    info!("{:} Level Up", weapon);

    Ok(())
}

pub fn spawn_scale_projectile(
    _trigger: On<WeaponAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    scale: Query<Entity, With<Scale>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let scale = scale.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

    commands
        .spawn((
            Name::new("scale projectile"),
            Sprite {
                image: asset_server.load("fx/scale.png"),
                ..default()
            },
            CastWeapon(scale),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.),
            Direction(direction),
            PlayerProjectile,
        ))
        .observe(on_scale_hit);

    Ok(())
}

fn on_scale_hit(
    event: On<CollisionStart>,
    mut commands: Commands,
    scale_dmg: Query<&Damage, With<Scale>>,
) -> Result {
    info!("hit detected");
    let projectile = event.collider1;
    let enemy = event.collider2;

    let dmg = scale_dmg.single()?.0;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy,
        dmg,
        damage_type: DamageType::Physical,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy,
        projectile,
    });

    commands.entity(projectile).despawn();
    Ok(())
}
