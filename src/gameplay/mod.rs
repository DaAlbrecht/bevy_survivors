use bevy::prelude::*;

use crate::gameplay::spells::SpellType;

pub mod enemy;
pub mod experience;
pub mod healthbar;
pub mod player;
pub mod spells;

#[derive(Component)]
pub(crate) struct Health(pub f32);

#[derive(Event)]
pub(crate) struct PickUpSpell {
    pub spell_type: SpellType,
}
