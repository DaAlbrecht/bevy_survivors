use bevy::prelude::*;

pub(crate) mod assets;
mod behaviours;
pub(crate) mod components;
pub(crate) mod kind;
pub(crate) mod systems;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, behaviours::plugin, systems::plugin));
}

pub(crate) trait ApplySpec {
    fn apply(&self, commands: &mut Commands, entity: Entity);
}
