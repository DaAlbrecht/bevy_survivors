use bevy::prelude::*;

use crate::gameplay::spells::SpellType;

pub mod enemy;
pub mod experience;
pub mod healthbar;
pub mod level;
pub mod player;
pub mod spells;

#[derive(Component, Reflect)]
pub(crate) struct Health(pub f32);

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

#[derive(Event, Reflect)]
pub(crate) struct PickUpSpell {
    pub spell_type: SpellType,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        enemy::plugin,
        player::plugin,
        experience::plugin,
        healthbar::plugin,
        spells::plugin,
        level::plugin,
    ));
}
