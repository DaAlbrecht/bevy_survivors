use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::{
    enemy::Speed,
    player::{Direction, Player, spawn_player},
};

use super::{Cooldown, Knockback, PlayerSpell};

use bevy_rand::{global::GlobalEntropy, prelude::WyRand};

pub struct ScalePlugin;

impl Plugin for ScalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scale.after(spawn_player));
        app.add_systems(Update, (spawn_scale_projectile, update_scale_timer));
    }
}

#[derive(Component, Reflect)]
pub struct Scale;

#[derive(Component, Reflect)]
pub struct ScaleProjectile;

//for now spawn allways, later on pick up or for default for chars
fn spawn_scale(mut commands: Commands) {
    commands.spawn((Scale, Cooldown(Timer::from_seconds(1.0, TimerMode::Once))));
}

fn spawn_scale_projectile(
    mut scale_cd_q: Query<&mut Cooldown, With<Scale>>,
    player_pos_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_pos_q.single()?;
    let mut scale_cd = scale_cd_q.single_mut()?;
    let random_angle: f32 = rng.gen_range(0.0..(2. * PI));

    if scale_cd.0.finished() {
        let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

        commands.spawn((
            Sprite {
                image: asset_server.load("Bullet.png"),
                ..default()
            },
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
            PlayerSpell,
            Speed(600.),
            Knockback(1500.),
            Direction(direction),
            ScaleProjectile,
            Visibility::Visible,
        ));

        // commands.entity(scale_entity).add_child(projectile_entity);
        scale_cd.0.reset();
    }

    Ok(())
}

fn update_scale_timer(time: Res<Time>, mut cooldowns: Query<&mut Cooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }
}
