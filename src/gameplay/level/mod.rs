use std::collections::HashSet;

use bevy::prelude::*;
use bevy_ecs_ldtk::{
    GridCoords, LdtkIntCell, LdtkProjectHandle, LdtkWorldBundle, LevelEvent, LevelSelection,
    app::LdtkIntCellAppExt,
    assets::{LdtkProject, LevelMetadataAccessor},
    utils::translation_to_grid_coords,
};
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{asset_tracking::LoadResource, audio::MusicPool, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
    app.init_resource::<LevelWalls>();
    app.register_ldtk_int_cell::<WallBundle>(1);
    app.insert_resource(LevelSelection::index(0));
    app.add_systems(Update, cache_wall_locations);
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

#[derive(Default, Component)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Default, Resource)]
pub struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    pub fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x < 0
            || grid_coords.y < 0
            || grid_coords.x >= self.level_width
            || grid_coords.y >= self.level_height
            || self.wall_locations.contains(grid_coords)
    }
}

fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_messages: MessageReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for level_event in level_messages.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_project_entities.single()?)
                .expect("LdtkProject should be loaded when level is spawned");
            let level = ldtk_project
                .get_raw_level_by_iid(level_iid.get())
                .expect("spawned level should exist in project");

            let wall_locations = walls.iter().copied().collect();

            let new_level_walls = LevelWalls {
                wall_locations,
                level_width: level.px_wid / 32,
                level_height: level.px_hei / 32,
            };

            *level_walls = new_level_walls;
        }
    }
    Ok(())
}

#[inline]
fn grid_to_world(coords: GridCoords, tile_size: f32) -> Vec2 {
    Vec2::new(
        coords.x as f32 * tile_size + tile_size * 0.5,
        coords.y as f32 * tile_size + tile_size * 0.5,
    )
}

/// Finds the nearest walkable tile inside level bounds and outside walls.
/// Returns world-space coordinates.
pub fn find_valid_spawn_position(
    desired_world_pos: Vec2,
    walls: &LevelWalls,
    tile_size: f32,
    max_search_radius: i32,
) -> Vec2 {
    // Convert world → grid
    let mut grid = translation_to_grid_coords(desired_world_pos, IVec2::splat(tile_size as i32));

    // Clamp inside level bounds
    grid.x = grid.x.clamp(0, walls.level_width - 1);
    grid.y = grid.y.clamp(0, walls.level_height - 1);

    // If free, use it
    if !walls.in_wall(&grid) {
        return grid_to_world(grid, tile_size);
    }

    // Search nearby tiles (square spiral)
    for radius in 1..=max_search_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let candidate = GridCoords {
                    x: grid.x + dx,
                    y: grid.y + dy,
                };

                // Skip out-of-bounds
                if candidate.x < 0
                    || candidate.y < 0
                    || candidate.x >= walls.level_width
                    || candidate.y >= walls.level_height
                {
                    continue;
                }

                // Accept if walkable
                if !walls.in_wall(&candidate) {
                    return grid_to_world(candidate, tile_size);
                }
            }
        }
    }

    // Fallback: return clamped tile even if it's a wall
    grid_to_world(grid, tile_size)
}

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
