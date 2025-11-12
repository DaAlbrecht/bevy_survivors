mod level_up;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Overlay>();

    app.add_plugins(level_up::plugin);
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Overlay {
    #[default]
    None,
    LevelUp,
}
