use avian2d::prelude::LinearVelocity;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_tiled::prelude::TiledMapAsset;
use bevy_enhanced_input::{EnhancedInputSystems, action::Action, prelude::InputAction};

use crate::{
    CAMERA_DECAY_RATE, GameplaySystems, PausableSystems, PostPhysicsAppSystems,
    fixed_update_inspection::did_fixed_update_happen,
    gameplay::{character_controller::CharacterController, player::Player},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        apply_movement
            .in_set(PausableSystems)
            .in_set(GameplaySystems::Movement),
    );

    app.add_systems(
        PreUpdate,
        (record_player_directional_input).after(EnhancedInputSystems::Apply),
    );

    app.add_systems(Update, clear_input.run_if(did_fixed_update_happen));

    app.add_systems(
        Update,
        translate_camera.in_set(PostPhysicsAppSystems::Update),
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
    pub last_move: Vec2,
    // Other input that could make sense would be e.g.
    // boost: bool
}

// Clear the input after it was processed in the fixed timestep.
fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}

fn record_player_directional_input(
    move_action: Single<&Action<Move>>,
    mut input: Single<&mut AccumulatedInput>,
) {
    input.last_move = move_action.normalize_or_zero();
}

/// Sync the camera's position with the player's interpolated position
fn translate_camera(
    time: Res<Time>,
    mut camera_transform: Single<&mut Transform, (With<Camera>, Without<Player>)>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    player: Single<&Transform, (With<Player>, Without<Camera>)>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let viewport_height = 504.;

    let aspect_ratio = window.width() / window.height();
    let viewport_width = viewport_height * aspect_ratio;

    // Get map dimensions from the first loaded Tiled map
    if let Some((_, tiled_map)) = tiled_map_assets.iter().next() {
        let level_width = tiled_map.map.width as f32 * tiled_map.map.tile_width as f32;
        let level_height = tiled_map.map.height as f32 * tiled_map.map.tile_height as f32;

        let mut desired_x = x - viewport_width / 2.0;
        let mut desired_y = y - viewport_height / 2.0;

        // Level bounds in WORLD space
        let min_x = 0.0;
        let max_x = level_width - viewport_width;

        let min_y = 0.0;
        let max_y = level_height - viewport_height;

        // Clamp in WORLD coordinates
        desired_x = desired_x.clamp(min_x, max_x);
        desired_y = desired_y.clamp(min_y, max_y);

        let target_pos = Vec3::new(desired_x, desired_y, camera_transform.translation.z);

        camera_transform.translation.smooth_nudge(
            &target_pos,
            CAMERA_DECAY_RATE,
            time.delta_secs(),
        );
    }
}

fn apply_movement(
    controller: Single<
        (&CharacterController, &mut LinearVelocity, &AccumulatedInput),
        With<Player>,
    >,
) {
    let (controller, mut linear_velocity, accumulated_input) = controller.into_inner();

    let velocity = accumulated_input.last_move * controller.speed;

    linear_velocity.x = velocity.x + controller.ability_velocity.x;
    linear_velocity.y = velocity.y + controller.ability_velocity.y;
}
