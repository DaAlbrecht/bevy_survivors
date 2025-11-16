use bevy::prelude::*;
use bevy_seedling::SeedlingPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(SeedlingPlugin::default());
}
