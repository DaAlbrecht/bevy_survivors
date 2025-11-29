use avian2d::prelude::*;
use bevy::prelude::*;

/// A marker component indicating that an entity is using a character controller.
#[derive(Component, Default)]
#[require(RigidBody::Dynamic, Collider)]
pub struct CharacterController {
    /// Max Movement speed multiplier: used to turn `velocity` into world units/sec.
    pub speed: f32,
    /// Additional velocity from abilities (dash, knockback, etc.) that gets added to movement.
    /// This is separate from player input and decays over time.
    /// TODO: Think about naming and if this belongs in our character controller.
    pub ability_velocity: Vec2,
}
