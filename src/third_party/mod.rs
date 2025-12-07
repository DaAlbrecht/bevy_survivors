//! All third party plugins are registered here.
//!
//! We use one file per plugin.

use bevy::prelude::*;

mod avian2d;
mod bevy_ecs_tiled;
mod bevy_enhanced_input;
mod bevy_rand;
mod bevy_seedling;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        bevy_ecs_tiled::plugin,
        bevy_enhanced_input::plugin,
        bevy_rand::plugin,
        bevy_seedling::plugin,
        avian2d::plugin,
    ));
}
