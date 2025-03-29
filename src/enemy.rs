use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};

use rand::Rng;

use crate::player::Player;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_enemy.run_if(on_timer(Duration::from_millis(500))),
        );
    }
}

const SPAWN_RADIUS: f32 = 200.0;

#[derive(Component)]
pub struct Enemy;

fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
) -> Result {
    let player_pos = player_query.single()?;
    let mut rng = rand::rng();

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    commands.spawn((
        Sprite {
            image: asset_server.load("Enemy.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        Enemy,
    ));

    Ok(())
}
