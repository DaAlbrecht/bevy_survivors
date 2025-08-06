use bevy::prelude::*;

pub mod attacks;
pub mod enemy;
pub mod experience;
pub mod healthbar;
pub mod player;

#[derive(Component)]
pub struct Health(pub f32);
