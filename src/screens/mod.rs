use bevy::prelude::*;

mod gameplay;
mod level_up;
mod title;

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default, Reflect)]
pub(crate) enum Screen {
    #[default]
    Title,
    Gameplay,
    LevelUp,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((gameplay::plugin, title::plugin, level_up::plugin));

    app.register_type::<Screen>();
}
