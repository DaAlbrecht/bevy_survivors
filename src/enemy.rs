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
        app.add_systems(Update, enemy_movement);
    }
}

const SPAWN_RADIUS: f32 = 200.0;
const SEPARATION_RADIUS: f32 = 40.;
const SEPARATION_FORCE: f32 = 10.;
// TODO: this should be a `Component` so different enemies can have different speeds;
const ENEMY_SPEED: f32 = 50.;

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

fn enemy_movement(
    enemy_transform_q: Query<&mut Transform, With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) -> Result {
    let player_transform = player_query.single()?;

    let enemy_positions = enemy_transform_q
        .iter()
        .map(|t| t.translation)
        .collect::<Vec<Vec3>>();

    for mut transform in enemy_transform_q {
        let direction = (player_transform.translation - transform.translation).normalize();

        // Separation force calculation for enemies
        let mut separation_force = Vec3::ZERO;
        for &other_pos in &enemy_positions {
            // skip ourselves
            if other_pos == transform.translation {
                continue;
            }
            // Check if the distance between enemy `A` and all other enemies is less than the
            // `SEPARATION_RADIUS`. If so, push enemy `A` away from the other enemy to maintain spacing.
            let distance = transform.translation.distance(other_pos);
            if distance < SEPARATION_RADIUS {
                let push_dir = (transform.translation - other_pos).normalize();
                let push_strength = (SEPARATION_RADIUS - distance) / SEPARATION_RADIUS;
                separation_force += push_dir * push_strength * SEPARATION_FORCE;
            }
        }
        // Separation force calculation for the player
        let distance_to_player = transform.translation.distance(player_transform.translation);
        if distance_to_player < SEPARATION_RADIUS {
            let push_dir = (transform.translation - player_transform.translation).normalize();
            let push_strength = (SEPARATION_RADIUS - distance_to_player) / SEPARATION_RADIUS;
            separation_force += push_dir * push_strength * SEPARATION_RADIUS;
        }

        let movement =
            (direction + separation_force).normalize() * (ENEMY_SPEED * time.delta_secs());
        transform.translation += movement;
    }
    Ok(())
}
