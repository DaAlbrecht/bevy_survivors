use bevy::prelude::*;

mod gameplay;
mod leevl_up;
mod title;

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default, Reflect)]
pub enum Screen {
    //#[default]
    Title,
    Gameplay,
    #[default]
    LevelUp,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((gameplay::plugin, title::plugin, leevl_up::plugin));

    app.register_type::<Screen>();
}
