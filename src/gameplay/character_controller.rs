use avian2d::prelude::*;
use bevy::prelude::*;

/// A marker component indicating that an entity is using a character controller.
#[derive(Component, Default)]
#[require(RigidBody::Dynamic, Collider)]
pub struct CharacterController {
    /// Max Movement speed multiplier: used to turn `velocity` into world units/sec.
    pub speed: f32,
}
