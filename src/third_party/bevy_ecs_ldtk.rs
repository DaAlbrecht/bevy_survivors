use bevy::prelude::*;
use bevy_ecs_ldtk::{IntGridRendering, LdtkPlugin, LdtkSettings};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);

    app.insert_resource(LdtkSettings {
        int_grid_rendering: IntGridRendering::Invisible,
        ..Default::default()
    });
}
