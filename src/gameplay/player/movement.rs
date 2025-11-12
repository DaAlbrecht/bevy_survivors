//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;
use bevy_enhanced_input::{action::Action, prelude::InputAction};

use crate::{
    CAMERA_DECAY_RATE, PausableSystems, PostPhysicsAppSystems, PrePhysicsAppSystems,
    fixed_update_inspection::did_fixed_update_happen,
    gameplay::{movement::MovementController, player::Player},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        (
            record_player_directional_input.in_set(PrePhysicsAppSystems::AccumulateInput),
            clear_input
                .in_set(PostPhysicsAppSystems::FixedTimestepDidRun)
                .run_if(did_fixed_update_happen),
            translate_camera.in_set(PostPhysicsAppSystems::UpdateCamera),
        )
            .in_set(PausableSystems),
    );
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub(crate) struct Move;

/// A vector representing the player's input, accumulated over all frames that ran
/// since the last time the physics simulation was advanced.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct AccumulatedInput {
    // The player's movement input (WASD).
    pub movement: Vec3,
    // Other input that could make sense would be e.g.
    // boost: bool
}

// Clear the input after it was processed in the fixed timestep.
fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}

fn record_player_directional_input(
    move_action: Single<&Action<Move>>,
    player: Single<(&mut AccumulatedInput, &mut MovementController)>,
) -> Result {
    let (mut input, mut controller) = player.into_inner();
    //
    // Collect directional input.
    input.movement = Vec3::ZERO;

    let mut dir = move_action.extend(0.0);
    dir = dir.normalize_or_zero();

    input.movement += dir;

    controller.velocity = input.movement;

    Ok(())
}

// Sync the camera's position with the player's interpolated position
fn translate_camera(
    time: Res<Time>,
    mut camera: Single<&mut Transform, With<Camera>>,
    player: Single<&Transform, (With<Player>, Without<Camera>)>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
