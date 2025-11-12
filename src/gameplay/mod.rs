use bevy::prelude::*;

use crate::gameplay::spells::SpellType;

mod movement;

pub(crate) mod enemy;
pub(crate) mod experience;
pub(crate) mod healthbar;
pub(crate) mod level;
pub(crate) mod overlays;
pub(crate) mod player;
pub(crate) mod spells;
pub(crate) mod waves;

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
        experience::plugin,
        healthbar::plugin,
        level::plugin,
        movement::plugin,
        overlays::plugin,
        player::plugin,
        spells::plugin,
        waves::plugin,
    ));
}
