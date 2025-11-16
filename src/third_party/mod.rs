//! All third party plugins are registered here.
//!
//! We use one file per plugin.

use bevy::prelude::*;

mod bevy_ecs_ldtk;
mod bevy_enhanced_input;
mod bevy_rand;
mod bevy_seedling;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        bevy_ecs_ldtk::plugin,
        bevy_enhanced_input::plugin,
        bevy_rand::plugin,
        bevy_seedling::plugin,
    ));
}
