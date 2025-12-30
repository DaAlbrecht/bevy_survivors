use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        character_controller::CharacterController,
        enemy::{DamageCooldown, Enemy, EnemyType, HitDamage, Meele},
        player::Player,
        simple_animation::{AnimationIndices, AnimationTimer},
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
    player_q: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    walker_stats: Res<WalkerStats>,
) {
    let Ok(player_pos) = player_q.single() else {
        return;
    };

    let stats = walker_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    let desired = Vec2::new(
        player_pos.translation.x + offset_x,
        player_pos.translation.y + offset_y,
    );

    let enemy_pos_x = desired.x;
    let enemy_pos_y = desired.y;

    let texture: Handle<Image> = asset_server.load(stats.sprite.clone());
    let layout = TextureAtlasLayout::from_grid(UVec2 { x: 58, y: 24 }, 11, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 10 };

    commands.spawn((
        Name::new("Walker"),
        Walker,
        HitDamage(stats.damage),
        Health(stats.health),
        Speed(stats.speed),
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.0)
            .with_scale(Vec3::splat((ENEMY_SIZE / 24.0) * 0.7)),
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        animation_indices,
        AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        },
        CharacterController {
            speed: stats.speed,
            ..default()
        },
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
        children![(
            Sprite {
                image: asset_server.load("fx/shadow.png"),

                ..Default::default()
            },
            Transform::from_xyz(0., -16.0, -0.1).with_scale(Vec3 {
                x: 4.,
                y: 1.,
                z: 1.
            })
        )],
    ));
}

fn patch_walker(trigger: On<WalkerPatchEvent>, mut stats: ResMut<WalkerStats>) {
    let (power_level, sprite) = (trigger.0, &trigger.1);
    stats.damage *= power_level;
    stats.health *= power_level;
    stats.speed += 10.0 * power_level;
    stats.sprite = sprite.clone();
}
