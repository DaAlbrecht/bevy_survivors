use bevy::{post_process::bloom::Bloom, prelude::*, render::view::Hdr};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use bevy_seedling::prelude::*;

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod fixed_update_inspection;
mod gameplay;
mod menus;
mod screens;
mod theme;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Window {
                    title: "bevy survivor".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        EnhancedInputPlugin,
        EntropyPlugin::<WyRand>::default(),
        SeedlingPlugin::default(),
    ));

    app.add_plugins((
        fixed_update_inspection::plugin,
        asset_tracking::plugin,
        audio::plugin,
        #[cfg(feature = "dev")]
        dev_tools::plugin,
        menus::plugin,
        theme::plugin,
        screens::plugin,
        gameplay::plugin,
    ));

    app.configure_sets(
        Update,
        (
            PostPhysicsAppSystems::TickTimers,
            PostPhysicsAppSystems::ChangeUi,
            PostPhysicsAppSystems::PlaySound,
            PostPhysicsAppSystems::PlayAnimations,
            PostPhysicsAppSystems::Update,
        )
            .chain(),
    );

    app.configure_sets(
        RunFixedMainLoop,
        (
            (PrePhysicsAppSystems::AccumulateInput,)
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            (
                PostPhysicsAppSystems::FixedTimestepDidRun,
                PostPhysicsAppSystems::InterpolateTransforms,
                PostPhysicsAppSystems::UpdateCamera,
                PostPhysicsAppSystems::UpdateAnimations,
            )
                .chain()
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        )
            .chain(),
    );

    app.configure_sets(
        FixedUpdate,
        (
            PhysicsAppSystems::PhysicsAdjustments,
            PhysicsAppSystems::AdvancePhysics,
            PhysicsAppSystems::PhysicsResolution,
        )
            .chain(),
    );

    // Set up the `Pause` state.
    app.init_state::<Pause>();
    app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
    app.configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))));
    app.configure_sets(
        RunFixedMainLoop,
        PausableSystems.run_if(in_state(Pause(false))),
    );

    app.add_systems(Startup, spawn_camera);
}

const ENEMY_SIZE: f32 = 30.0;
const PLAYER_SIZE: f32 = 30.0;
const SPELL_SIZE: f32 = 16.0;
const XP_GAIN_GEM: f32 = 10.;

const SPAWN_RADIUS: f32 = 1000.0;
const SPAWN_RADIUS_BUFFER: f32 = 200.0;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum PrePhysicsAppSystems {
    AccumulateInput,
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum PhysicsAppSystems {
    PhysicsAdjustments,
    AdvancePhysics,
    PhysicsResolution,
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum PostPhysicsAppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    ChangeUi,
    /// Play sound
    PlaySound,
    /// FixedTimestepDidRun
    FixedTimestepDidRun,
    /// Interpolate
    InterpolateTransforms,
    /// Camera follow
    UpdateCamera,
    /// UpdateAnimations
    UpdateAnimations,
    /// Play animations.
    PlayAnimations,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Hdr,
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: (1000.0),
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
