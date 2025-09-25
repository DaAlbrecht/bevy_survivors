use bevy::prelude::*;

use crate::screens::Screen;
//use bevy::prelude::*;
//use bevy_ecs_tilemap::prelude::*;
//
//const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 32.0, y: 32.0 };
//const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };
//
// Render chunk sizes are set to 4 render chunks per user specified chunk.
//const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
//    x: CHUNK_SIZE.x * 2,
//    y: CHUNK_SIZE.y * 2,
//};
//
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, handle_game_timer);
}

fn handle_game_timer(mut time: ResMut<Time<Virtual>>, scren: Res<State<Screen>>) {
    match scren.get() {
        Screen::Gameplay => time.unpause(),
        _ => time.pause(),
    }
}
