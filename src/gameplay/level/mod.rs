use avian2d::prelude::{CollisionLayers, RigidBody};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_tiled::prelude::{ColliderCreated, TiledEvent, TiledMap, TiledMapAsset};
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{AssetStates, GameLayer, audio::MusicPool, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AssetStates::AssetLoading).load_collection::<LevelAssets>(),
    );
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
        );
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(AssetCollection, Resource)]
pub(crate) struct LevelAssets {
    #[asset(path = "level/winter/tiled/winter.tmx")]
    pub(crate) level: Handle<TiledMapAsset>,
    #[asset(path = "audio/music/city.ogg")]
    pub(crate) music: Handle<AudioSample>,
}
