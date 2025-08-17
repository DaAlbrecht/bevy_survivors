use bevy::prelude::*;

use crate::gameplay::attacks::SpellType;

pub mod attacks;
pub mod enemy;
pub mod experience;
pub mod healthbar;
pub mod player;

#[derive(Component)]
pub(crate) struct Health(pub f32);

#[derive(Event)]
pub(crate) struct PickUpSpell {
    pub spell_type: SpellType,
}
