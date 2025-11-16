use bevy::prelude::*;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EntropyPlugin::<WyRand>::default());
}
