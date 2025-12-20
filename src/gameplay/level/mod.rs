use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::{LdtkProjectHandle, LdtkWorldBundle, LevelSelection, assets::LdtkProject};
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{AssetStates, audio::MusicPool, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AssetStates::AssetLoading).load_collection::<LevelAssets>(),
    );
    app.insert_resource(LevelSelection::index(0));
}

pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    commands.spawn((
        Name::new("Level"),
        DespawnOnExit(Screen::Gameplay),
        LdtkWorldBundle {
            ldtk_handle: LdtkProjectHandle {
                handle: level_assets.level.clone(),
            },
            ..Default::default()
        },
        children![(
            Name::new("Gameplay Music"),
            SamplePlayer::new(level_assets.music.clone()).looping(),
            MusicPool
        )],
    ));
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Level;

#[derive(AssetCollection, Resource)]
pub(crate) struct LevelAssets {
    #[asset(path = "level/dungeon.ldtk")]
    pub(crate) level: Handle<LdtkProject>,
    #[asset(path = "audio/music/city.ogg")]
    pub(crate) music: Handle<AudioSample>,
}
