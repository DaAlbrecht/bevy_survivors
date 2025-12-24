#![allow(dead_code)]

pub(crate) mod animation;
pub(crate) mod interaction;
pub(crate) mod palette;
pub(crate) mod widget;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::{interaction::InteractionPalette, palette as ui_palette, widget};
}

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((interaction::plugin, animation::plugin));
}
