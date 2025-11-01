use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        enemy::{DamageCooldown, Enemy, EnemyType, KnockbackDirection, Meele},
        player::{Direction, Player},
        spells::{Damage, Knockback},
    },
};

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(WalkerStats {
        health: 10.0,
        damage: 2.0,
        speed: 50.0,
    });
    app.add_observer(spawn_walker).add_observer(patch_walker);
}

#[derive(Component)]
#[require(
    EnemyType::Walker,
    Meele,
    Enemy,
    KnockbackDirection(Direction(Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    })),
    Knockback(0.0),
)]
#[derive(Reflect)]
pub(crate) struct Walker;

#[derive(Resource)]
pub(crate) struct WalkerStats {
    health: f32,
    damage: f32,
    speed: f32,
}

#[derive(Event)]
pub(crate) struct WalkerSpawnEvent;

#[derive(Event)]
pub(crate) struct WalkerPatchEvent(pub f32);

fn spawn_walker(
    _trigger: On<WalkerSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    walker_stats: Res<WalkerStats>,
) -> Result {
    let player_pos = player_query.single()?;
    let stats = walker_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    commands.spawn((
        Name::new("Default Enemy"),
        Walker,
        Sprite {
            image: asset_server.load("enemies/walker.png"),
            ..default()
        },
        Damage(stats.damage),
        Health(stats.health),
        Speed(stats.speed),
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));

    Ok(())
}

fn patch_walker(trigger: On<WalkerPatchEvent>, mut stats: ResMut<WalkerStats>) {
    let power_level = trigger.0;
    stats.damage *= power_level;
    stats.health *= power_level;
    stats.speed += 10.0 * power_level;
}
