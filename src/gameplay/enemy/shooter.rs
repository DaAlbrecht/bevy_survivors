use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        enemy::{
            AbilityDamage, DamageCooldown, Enemy, EnemyProjectile, EnemyType, ProjectileOf, Ranged,
        },
        level::{LevelWalls, find_valid_spawn_position},
        movement::{MovementController, PhysicalTranslation, PreviousPhysicalTranslation},
        player::{Player, PlayerHitEvent},
        spells::{Cooldown, Damage, Range},
    },
};

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(ShooterStats {
        health: 10.0,
        damage: 1.0,
        ability_damage: 5.0,
        projectile_speed: 25.0,
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
    player_q: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    shooter_q: Query<&Shooter>,
    shooter_stats: Res<ShooterStats>,
    level_walls: Res<LevelWalls>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let stats = shooter_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    // tile size, search radius
    let desired = Vec2::new(
        player_pos.translation.x + offset_x,
        player_pos.translation.y + offset_y,
    );
    let adjusted_pos = find_valid_spawn_position(desired, &level_walls, 32.0, 8);

    let enemy_pos_x = adjusted_pos.x;
    let enemy_pos_y = adjusted_pos.y;

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
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 10.0)
            .with_scale(Vec3::splat(ENEMY_SIZE / 32.0)),
        PhysicalTranslation(Vec3::new(enemy_pos_x, enemy_pos_y, 10.0)),
        PreviousPhysicalTranslation(Vec3::new(enemy_pos_x, enemy_pos_y, 10.0)),
        MovementController {
            speed: 30.0,
            ..default()
        },
        Health(stats.health),
        Damage(stats.damage),
        AbilityDamage(stats.ability_damage),
        Range(stats.range),
        Cooldown(Timer::from_seconds(stats.cooldown, TimerMode::Repeating)),
        children![(
            Sprite {
                image: asset_server.load("shadow.png"),

                ..Default::default()
            },
            Transform::from_xyz(0., -16.0, -0.1).with_scale(Vec3 {
                x: 2.,
                y: 1.,
                z: 1.
            })
        )],
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
    shooter_q: Query<&PhysicalTranslation, With<Shooter>>,
    player_q: Query<&PhysicalTranslation, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let shooter = trigger.0;

    let Ok(shooter_pos) = shooter_q.get(shooter) else {
        return Ok(());
    };

    let direction = (player_pos.0 - shooter_pos.0).normalize();
    let towards_quaternion = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

    commands.spawn((
        EnemyProjectile,
        Sprite {
            image: asset_server.load("enemies/shooter_bullet.png"),
            ..default()
        },
        Transform::from_translation(shooter_pos.0)
            .with_rotation(towards_quaternion)
            .with_scale(Vec3::splat(0.5)),
        PhysicalTranslation(shooter_pos.0),
        PreviousPhysicalTranslation(shooter_pos.0),
        MovementController {
            velocity: direction,
            speed: 125.0,
            ..default()
        },
        ProjectileOf(shooter),
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
