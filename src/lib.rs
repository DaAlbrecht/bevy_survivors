use avian2d::prelude::PhysicsLayer;
use bevy::{camera::ScalingMode, prelude::*};
use bevy_asset_loader::prelude::*;

mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod fixed_update_inspection;
mod gameplay;
mod menus;
mod screens;
mod theme;
mod third_party;

pub fn plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Window {
                #[cfg(feature = "dev")]
                title: "float".to_string(),
                #[cfg(not(feature = "dev"))]
                title: "bevy survivor".to_string(),
                fit_canvas_to_parent: true,
                ..default()
            }
            .into(),
            ..default()
        })
        .set(ImagePlugin::default_nearest()),));

    // Setup the loading state
    app.init_state::<AssetStates>().add_loading_state(
        LoadingState::new(AssetStates::AssetLoading).continue_to_state(AssetStates::Next),
    );

    // Add all third party plugins.
    app.add_plugins(third_party::plugin);

    // Add all first party plugins.
    app.add_plugins((
        fixed_update_inspection::plugin,
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
        FixedUpdate,
        (GameplaySystems::MovementModify, GameplaySystems::Movement).chain(),
    );

    // Set up the `Pause` state.
    app.init_state::<Pause>();
    app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
    app.configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))));

    app.add_systems(Startup, spawn_camera);
}

const ENEMY_SIZE: f32 = 32.0;
const PLAYER_SIZE: f32 = 32.0;
const PROJECTILE_SIZE: f32 = 8.0;
const XP_GAIN_GEM: f32 = 10.;

const SPAWN_RADIUS: f32 = 200.0;
const SPAWN_RADIUS_BUFFER: f32 = 80.0;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

/// High-level groupings of systems for gameplay in the `FixedUpdate` schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) enum GameplaySystems {
    /// Things that modify velocity
    MovementModify,
    /// Movement that applies velocity to entities
    Movement,
}

/// High-level groupings of systems for the app in the `Update` schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum PostPhysicsAppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    ChangeUi,
    /// Play sound
    PlaySound,
    /// Play animations.
    PlayAnimations,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

#[derive(PhysicsLayer, Default)]
pub(crate) enum GameLayer {
    #[default]
    // Layer 0 - the default layer that objects are assigned to
    Default,
    // Layer 1
    Player,
    // Layer 2
    PlayerProjectiles,
    // Layer 3
    Enemy,
    // Layer 4
    EnemyProjectiles,
}

/// Whether we are still loading
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetStates {
    #[default]
    AssetLoading,
    Next,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

const VIRTUAL_W: f32 = 1024.0;
const VIRTUAL_H: f32 = 576.0;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: VIRTUAL_W,
                height: VIRTUAL_H,
            },
            viewport_origin: Vec2::ZERO,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
