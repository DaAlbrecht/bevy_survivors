use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystems,
    gameplay::{
        Health,
        enemy::{DamageCooldown, Enemy, KnockbackDirection, SPAWN_RADIUS, Speed},
        player::{Direction, Player},
        spells::Knockback,
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_walker
            .run_if(on_timer(Duration::from_millis(2000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Component)]
#[require(
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

fn spawn_walker(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    commands.spawn((
        Name::new("Default Enemy"),
        Walker,
        Sprite {
            image: asset_server.load("enemies/Walker.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));

    Ok(())
}
