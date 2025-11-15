use bevy::prelude::*;
use bevy_ecs_ldtk::{
    LdtkEntity, LdtkProjectHandle, LdtkWorldBundle, LevelSelection, app::LdtkEntityAppExt,
    assets::LdtkProject,
};
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{asset_tracking::LoadResource, audio::MusicPool, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
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

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelAssets {
    #[dependency]
    pub(crate) level: Handle<LdtkProject>,
    #[dependency]
    pub(crate) music: Handle<AudioSample>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            level: assets.load("level/dungeon.ldtk"),
            music: assets.load("audio/music/city.ogg"),
        }
    }
}
