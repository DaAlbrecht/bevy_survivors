use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        enemy::{
            AbilityDamage, DamageCooldown, Enemy, EnemyProjectile, EnemyType, ProjectileOf,
            ProjectileSpeed, Ranged,
        },
        player::{Direction, Player, PlayerHitEvent},
        spells::{Cooldown, Damage, Range},
    },
};

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(ShooterStats {
        health: 10.0,
        damage: 1.0,
        ability_damage: 5.0,
        projectile_speed: 125.0,
        range: 200.0,
        cooldown: 2.0,
        sprite: "enemies/shooter.png".to_string(),
    });
    app.add_observer(spawn_shooter)
        .add_observer(shooter_attack)
        .add_observer(shooter_projectile_hit)
        .add_observer(patch_shooter);
}

#[derive(Component)]
#[require(
    EnemyType::Shooter,
    Ranged,
    Speed(100.),
    //Meele hit
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
)]
pub(crate) struct Shooter;

#[derive(Resource)]
pub(crate) struct ShooterStats {
    health: f32,
    damage: f32,
    ability_damage: f32,
    projectile_speed: f32,
    range: f32,
    cooldown: f32,
    sprite: String,
}

#[derive(Event)]
pub(crate) struct ShooterAttackEvent(pub Entity);

#[derive(Event)]
pub(crate) struct ShooterProjectileHitEvent {
    pub projectile: Entity,
    pub source: Entity,
}

#[derive(Event)]
pub(crate) struct ShooterSpawnEvent;

#[derive(Event)]
pub(crate) struct ShooterPatchEvent(pub f32, pub String);

fn spawn_shooter(
    _trigger: On<ShooterSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    shooter_q: Query<&Shooter>,
    shooter_stats: Res<ShooterStats>,
) -> Result {
    let player_pos = player_query.single()?;
    let stats = shooter_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let mut shooter_count = shooter_q.iter().count();
    shooter_count += 1;

    commands.spawn((
        Name::new(format!("Shooter {shooter_count}")),
        Enemy,
        Shooter,
        Sprite {
            image: asset_server.load(stats.sprite.clone()),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        Health(stats.health),
        Damage(stats.damage),
        AbilityDamage(stats.ability_damage),
        ProjectileSpeed(stats.projectile_speed),
        Range(stats.range),
        Cooldown(Timer::from_seconds(stats.cooldown, TimerMode::Once)),
    ));

    Ok(())
}

fn patch_shooter(trigger: On<ShooterPatchEvent>, mut stats: ResMut<ShooterStats>) {
    let (power_level, sprite) = (trigger.0, &trigger.1);

    stats.health *= power_level;
    stats.damage *= power_level;
    stats.ability_damage *= power_level;
    stats.projectile_speed += 50.0 * power_level;
    stats.range += 50.0 * power_level;
    stats.cooldown -= 0.1 * power_level;
    stats.sprite = sprite.clone();
}

fn shooter_attack(
    trigger: On<ShooterAttackEvent>,
    shooter_q: Query<&Transform, With<Shooter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let shooter = trigger.0;
    let player_pos = player_q.single()?.translation.truncate();

    let Ok(transform) = shooter_q.get(shooter) else {
        return Ok(());
    };

    let shooter_pos = transform.translation.truncate();
    let direction = (player_pos - shooter_pos).normalize();
    let angle = direction.y.atan2(direction.x);

    commands.spawn((
        Sprite {
            image: asset_server.load("enemies/shooter_bullet.png"),
            ..default()
        },
        Transform {
            translation: transform.translation,
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        EnemyProjectile,
        ProjectileOf(shooter),
        Direction(direction.extend(0.0)),
    ));

    Ok(())
}

fn shooter_projectile_hit(
    trigger: On<ShooterProjectileHitEvent>,
    shooter_q: Query<&AbilityDamage, With<Shooter>>,
    mut commands: Commands,
) {
    let projectile = trigger.projectile;
    let shooter = trigger.source;

    let Ok(damage) = shooter_q.get(shooter) else {
        return;
    };

    commands.trigger(PlayerHitEvent { dmg: damage.0 });

    commands.entity(projectile).despawn();
}
