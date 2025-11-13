use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        enemy::{DamageCooldown, Enemy, EnemyType, Meele},
        movement::{MovementController, PhysicalTranslation, PreviousPhysicalTranslation},
        player::Player,
        spells::Damage,
    },
};

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(WalkerStats {
        health: 10.0,
        damage: 2.0,
        speed: 30.0,
        sprite: "enemies/walker.png".to_string(),
    });
    app.add_observer(spawn_walker).add_observer(patch_walker);
}

#[derive(Component)]
#[require(EnemyType::Walker, Meele, Enemy)]
#[derive(Reflect)]
pub(crate) struct Walker;

#[derive(Resource)]
pub(crate) struct WalkerStats {
    health: f32,
    damage: f32,
    speed: f32,
    sprite: String,
}

#[derive(Event)]
pub(crate) struct WalkerSpawnEvent;

#[derive(Event)]
pub(crate) struct WalkerPatchEvent(pub f32, pub String);

fn spawn_walker(
    _trigger: On<WalkerSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_q: Query<&PhysicalTranslation, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    walker_stats: Res<WalkerStats>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let stats = walker_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    let enemy_pos_x = player_pos.x + offset_x;
    let enemy_pos_y = player_pos.y + offset_y;

    commands.spawn((
        Name::new("Walker"),
        Walker,
        Sprite {
            image: asset_server.load(stats.sprite.clone()),
            ..default()
        },
        Damage(stats.damage),
        Health(stats.health),
        Speed(stats.speed),
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 10.0)
            .with_scale(Vec3::splat(ENEMY_SIZE / 32.0)),
        PhysicalTranslation(Vec3::new(enemy_pos_x, enemy_pos_y, 10.)),
        PreviousPhysicalTranslation(Vec3::new(enemy_pos_x, enemy_pos_y, 10.)),
        MovementController {
            speed: stats.speed,
            ..default()
        },
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));

    Ok(())
}

fn patch_walker(trigger: On<WalkerPatchEvent>, mut stats: ResMut<WalkerStats>) {
    let (power_level, sprite) = (trigger.0, &trigger.1);
    stats.damage *= power_level;
    stats.health *= power_level;
    stats.speed += 10.0 * power_level;
    stats.sprite = sprite.clone();
}
