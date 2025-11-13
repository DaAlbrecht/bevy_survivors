use bevy::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{
    asset_tracking::LoadResource,
    audio::MusicPool,
    gameplay::{
        healthbar::HealthBarMaterial,
        player::{PlayerAssets, player},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(
                400.0,
                &player_assets,
                &mut texture_atlas_layouts,
                &mut health_bar_materials,
                &mut mesh,
            ),
            (
                Name::new("Gameplay Music"),
                SamplePlayer::new(level_assets.music.clone()).looping(),
                MusicPool
            )
        ],
    ));

    commands.trigger(crate::gameplay::PickUpSpell {
        spell_type: crate::gameplay::spells::SpellType::Fireball,
    });
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Level;

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelAssets {
    // #[dependency]
    // pub(crate) level: TILEMAP
    #[dependency]
    pub(crate) music: Handle<AudioSample>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            music: assets.load("audio/music/city.ogg"),
        }
    }
}
