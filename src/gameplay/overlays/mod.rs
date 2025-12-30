pub(crate) mod experience;
mod hud;
mod level_up;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Overlay>();

    app.add_plugins((experience::plugin, level_up::plugin, hud::plugin));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Overlay {
    #[default]
    None,
    LevelUp,
}
