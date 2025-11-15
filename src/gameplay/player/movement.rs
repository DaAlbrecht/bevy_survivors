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

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_ldtk::{
    LdtkProjectHandle, LevelIid, LevelSelection,
    assets::{LdtkProject, LevelIndices, LevelMetadataAccessor},
};
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
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    level_query: Query<(&Transform, &LevelIid), (Without<Player>, Without<Camera>)>,
    ldtk_projects: Query<&LdtkProjectHandle>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    player: Single<&Transform, (With<Player>, Without<Camera>)>,
    window: Single<&Window, With<PrimaryWindow>>,
) -> Result {
    let Vec3 { x, y, .. } = player.translation;
    let mut camera_transform = camera_query.single_mut()?;
    let viewport_height = 504.;

    let aspect_ratio = window.width() / window.height();
    let viewport_width = viewport_height * aspect_ratio;

    for (level_transform, level_iid) in &level_query {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single()?)
            .expect("Project should be loaded if level has spawned");

        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        if !level_selection.is_match(&LevelIndices::default(), level) {
            continue;
        }

        let level_origin = level_transform.translation;
        let level_width = level.px_wid as f32;
        let level_height = level.px_hei as f32;

        let mut desired_x = x - viewport_width / 2.0;
        let mut desired_y = y - viewport_height / 2.0;

        // Level bounds in WORLD space
        let min_x = level_origin.x;
        let max_x = level_origin.x + level_width - viewport_width;

        let min_y = level_origin.y;
        let max_y = level_origin.y + level_height - viewport_height;

        // Clamp in WORLD coordinates
        desired_x = desired_x.clamp(min_x, max_x);
        desired_y = desired_y.clamp(min_y, max_y);

        let target_pos = Vec3::new(
            level_origin.x + desired_x,
            level_origin.y + desired_y,
            camera_transform.translation.z, // keep z
        );

        camera_transform.translation.smooth_nudge(
            &target_pos,
            CAMERA_DECAY_RATE,
            time.delta_secs(),
        );
    }
    Ok(())
}
