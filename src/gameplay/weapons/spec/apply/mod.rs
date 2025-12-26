use bevy::prelude::*;

pub(crate) mod attacks;
pub(crate) mod sfx;
pub(crate) mod visuals;

pub trait ApplySpec {
    fn apply(&self, commands: &mut Commands, entity: Entity);
}
