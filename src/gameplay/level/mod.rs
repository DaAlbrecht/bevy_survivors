use avian2d::prelude::{CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{ColliderCreated, MapCreated, TiledEvent, TiledMap, TiledMapAsset};
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{
    GameLayer, asset_tracking::LoadResource, audio::MusicPool, gameplay::player::Player,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    commands
        .spawn((
            Name::new("Level"),
            DespawnOnExit(Screen::Gameplay),
            TiledMap(level_assets.level.clone()),
            children![(
                Name::new("Gameplay Music"),
                SamplePlayer::new(level_assets.music.clone()).looping(),
                MusicPool
            )],
        ))
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands.entity(collider_created.event().origin).insert((
                    RigidBody::Static,
                    CollisionLayers::new(GameLayer::Default, [GameLayer::Player, GameLayer::Enemy]),
                ));
            },
        )
        .observe(|_: On<TiledEvent<MapCreated>>, mut commands: Commands| {
            commands.spawn((Player, Transform::from_xyz(100.0, 100.0, 0.0)));
        });
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelAssets {
    #[dependency]
    pub(crate) level: Handle<TiledMapAsset>,
    #[dependency]
    pub(crate) music: Handle<AudioSample>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            level: assets.load("level/winter/tiled/winter.tmx"),
            music: assets.load("audio/music/city.ogg"),
        }
    }
}
