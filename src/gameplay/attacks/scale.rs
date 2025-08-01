use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::{
    attacks::{Attack, SpellType},
    enemy::Speed,
    player::{Direction, Player, spawn_player},
};

use super::{Cooldown, Knockback, PlayerProjectile};

use bevy_rand::{global::GlobalEntropy, prelude::WyRand};

#[derive(Event)]
pub struct ScaleAttackEvent;

pub struct ScalePlugin;

impl Plugin for ScalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_scale).after(spawn_player));

        app.add_systems(Update, (update_scale_timer,));

        app.add_observer(spawn_scale_projectile);
    }
}

//for now spawn allways, later on pick up or default for starting item
fn spawn_scale(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    let scale = commands
        .spawn((
            Attack,
            SpellType::Scale,
            Cooldown(Timer::from_seconds(1.0, TimerMode::Once)),
            Name::new("Scale"),
        ))
        .id();

    commands.entity(player).add_child(scale);

    Ok(())
}

fn spawn_scale_projectile(
    _trigger: Trigger<ScaleAttackEvent>,
    player_pos_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_pos_q.single()?;
    let random_angle: f32 = rng.gen_range(0.0..(2. * PI));

    let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

    commands.spawn((
        Sprite {
            image: asset_server.load("Bullet.png"),
            ..default()
        },
        Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
        PlayerProjectile,
        Speed(600.),
        Knockback(1500.),
        Direction(direction),
        Name::new("ScaleProjectile"),
    ));

    Ok(())
}

fn update_scale_timer(time: Res<Time>, mut cooldowns: Query<&mut Cooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }
}
