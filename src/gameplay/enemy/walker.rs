use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        enemy::{DamageCooldown, Enemy, EnemyType, Meele},
        level::{LevelWalls, find_valid_spawn_position},
        movement::{MovementController, PhysicalTranslation, PreviousPhysicalTranslation},
        player::Player,
        simple_animation::{AnimationIndices, AnimationTimer},
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
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    walker_stats: Res<WalkerStats>,
    level_walls: Res<LevelWalls>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let stats = walker_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    let desired = Vec2::new(player_pos.x + offset_x, player_pos.y + offset_y);

    // tile size, search radius
    let adjusted_pos = find_valid_spawn_position(desired, &level_walls, 32.0, 8);

    let enemy_pos_x = adjusted_pos.x;
    let enemy_pos_y = adjusted_pos.y;

    let texture: Handle<Image> = asset_server.load(stats.sprite.clone());
    let layout = TextureAtlasLayout::from_grid(UVec2 { x: 90, y: 64 }, 10, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        Name::new("Walker"),
        Walker,
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
        Damage(stats.damage),
        Health(stats.health),
        Speed(stats.speed),
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 10.0),
        PhysicalTranslation(Vec3::new(enemy_pos_x, enemy_pos_y, 10.)),
        PreviousPhysicalTranslation(Vec3::new(enemy_pos_x, enemy_pos_y, 10.)),
        MovementController {
            speed: stats.speed,
            mass: 100.,
            ..default()
        },
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
        children![(
            Sprite {
                image: asset_server.load("shadow.png"),

                ..Default::default()
            },
            Transform::from_xyz(0., -32.0, -0.1).with_scale(Vec3 {
                x: 4.,
                y: 1.,
                z: 1.
            })
        )],
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
