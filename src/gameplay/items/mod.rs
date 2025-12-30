use bevy::prelude::*;

pub(crate) mod inventory;
pub(crate) mod items;
pub(crate) mod loader;
pub(crate) mod stats;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((stats::plugin, items::plugin, loader::plugin));
}
