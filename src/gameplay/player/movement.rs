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

use crate::{CAMERA_DECAY_RATE, PausableSystems, PostPhysicsAppSystems, gameplay::player::Player};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        (translate_camera)
            .in_set(PostPhysicsAppSystems::UpdateCamera)
            .in_set(PausableSystems),
    );
}

// Sync the camera's position with the player's interpolated position
fn translate_camera(
    fixed_time: Res<Time<Fixed>>,
    mut camera: Single<&mut Transform, With<Camera>>,
    player: Single<&Transform, (With<Player>, Without<Camera>)>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, fixed_time.delta_secs());
}
