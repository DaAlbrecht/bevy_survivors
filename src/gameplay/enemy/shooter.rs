use avian2d::prelude::*;
use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    GameLayer, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        character_controller::CharacterController,
        enemy::{
            AbilityDamage, DamageCooldown, Enemy, EnemyProjectile, EnemyType, ProjectileOf, Ranged,
        },
        player::{Direction, Player, PlayerHitEvent},
        weapons::{Cooldown, Damage, Range},
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
) {
    let Ok(player_pos) = player_q.single() else {
        return;
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

    let enemy_pos_x = desired.x;
    let enemy_pos_y = desired.y;

    let mut shooter_count = shooter_q.iter().count();
    shooter_count += 1;

    commands.spawn((
        Name::new(format!("Shooter {shooter_count}")),
        Enemy,
        Shooter,
        // Overwrite default Collider size
        Collider::default(),
        children![
            (
                Collider::rectangle(32., 16.),
                //Reapply the collision layer
                CollisionLayers::new(
                    GameLayer::Enemy,
                    [
                        GameLayer::Player,
                        GameLayer::Default,
                        GameLayer::PlayerProjectiles
                    ]
                ),
                Transform::from_xyz(0., -6., 0.0)
            ),
            (
                Sprite {
                    image: asset_server.load("fx/shadow.png"),

                    ..Default::default()
                },
                Transform::from_xyz(0., -16.0, -0.1).with_scale(Vec3 {
                    x: 4.,
                    y: 1.,
                    z: 1.
                })
            )
        ],
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 10.0),
        Sprite {
            image: asset_server.load(stats.sprite.clone()),
            ..default()
        },
        CharacterController {
            speed: 30.0,
            ..default()
        },
        Health(stats.health),
        Damage(stats.damage),
        AbilityDamage(stats.ability_damage),
        Range(stats.range),
        Cooldown(Timer::from_seconds(stats.cooldown, TimerMode::Repeating)),
    ));
}

fn patch_shooter(trigger: On<ShooterPatchEvent>, mut stats: ResMut<ShooterStats>) {
    let (power_level, sprite) = (trigger.0, &trigger.1);

    stats.health *= power_level;
    stats.damage *= power_level;
    stats.ability_damage *= power_level;
    stats.projectile_speed += 50.0 * power_level;
    stats.range += 50.0 * power_level;
    stats.cooldown -= 0.1 * power_level;
    stats.sprite.clone_from(sprite);
}

fn shooter_attack(
    trigger: On<ShooterAttackEvent>,
    shooter_q: Query<&Transform, With<Shooter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player_q.single()?;
    let shooter = trigger.0;

    let shooter_pos = shooter_q.get(shooter)?;

    let direction = (player_pos.translation - shooter_pos.translation).normalize();
    let towards_quaternion = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

    commands.spawn((
        EnemyProjectile,
        Speed(80.),
        Direction(direction),
        Sprite {
            image: asset_server.load("enemies/shooter_bullet.png"),
            ..default()
        },
        Transform::from_translation(shooter_pos.translation)
            .with_rotation(towards_quaternion)
            .with_scale(Vec3::splat(0.5)),
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
