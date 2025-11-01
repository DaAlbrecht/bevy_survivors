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
        spells::Knockback,
    },
};

pub(crate) fn plugin(app: &mut App) {
    // app.add_systems(
    //     Update,
    //     spawn_walker
    //         .run_if(on_timer(Duration::from_millis(2000)))
    //         .run_if(in_state(Screen::Gameplay))
    //         .in_set(AppSystems::Update),
    // );
    app.add_observer(spawn_walker);
}

#[derive(Component)]
#[require(
    EnemyType::Walker,
    Meele,
    Health(10.),
    Speed(50.),
    DamageCooldown,
    Transform,
    KnockbackDirection(Direction(Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        })),
    Knockback(0.0),
    Enemy,
)]
#[derive(Reflect)]
pub(crate) struct Walker;

#[derive(Event)]
pub(crate) struct WalkerSpawnEvent;

fn spawn_walker(
    _trigger: On<WalkerSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let player_pos = player_query.single()?;

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
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));

    Ok(())
}
