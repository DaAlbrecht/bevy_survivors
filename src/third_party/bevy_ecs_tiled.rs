use bevy::prelude::*;
use bevy_ecs_tiled::{
    prelude::{TiledPhysicsAvianBackend, TiledPhysicsPlugin},
    tiled::TiledPlugin,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TiledPlugin::default())
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
}
