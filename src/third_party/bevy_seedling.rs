use bevy::prelude::*;
use bevy_seedling::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(SeedlingPlugins);
}
