use bevy::prelude::*;

use crate::{
    PausableSystems, PhysicsAppSystems, PostPhysicsAppSystems,
    fixed_update_inspection::did_fixed_update_happen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        advance_physics
            .in_set(PhysicsAppSystems::AdvancePhysics)
            .in_set(PausableSystems),
    );

    app.add_systems(
        RunFixedMainLoop,
        (interpolate_rendered_transform)
            .chain()
            .in_set(PostPhysicsAppSystems::InterpolateTransforms)
            .in_set(PausableSystems),
    );

    app.add_systems(PreUpdate, clear_intent.run_if(did_fixed_update_happen));
}
///
/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec3,

    pub physical_translation: Vec3,

    pub previous_physical_translation: Vec3,

    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec3::ZERO,
            physical_translation: Vec3::ZERO,
            previous_physical_translation: Vec3::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            speed: 400.0,
        }
    }
}

fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    mut controller_query: Query<&mut MovementController>,
) {
    for mut controller in &mut controller_query {
        controller.previous_physical_translation = controller.physical_translation;

        let velocity = controller.intent * controller.speed;
        controller.physical_translation += velocity * fixed_time.delta_secs();
    }
}

fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut controller_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut controller_query {
        let previous = controller.previous_physical_translation;
        let current = controller.physical_translation;
        // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
        let alpha = fixed_time.overstep_fraction();

        let rendered_translation = previous.lerp(current, alpha);
        transform.translation = rendered_translation;
    }
}

fn clear_intent(mut controller_query: Query<&mut MovementController>) {
    for mut controller in &mut controller_query {
        controller.intent = Vec3::ZERO;
    }
}
