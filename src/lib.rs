use bevy::prelude::*;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use enemy::EnemyPlugin;
use experience::ExperiencePlugin;
use healthbar::HealthBarPlugin;
use movement::MovementPlugin;
use player::{Player, PlayerPlugin};
use screens::Screen;

#[cfg(feature = "dev")]
mod dev_tools;
mod enemy;
mod experience;
mod healthbar;
mod movement;
mod player;
mod screens;
pub mod widgets;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (AppSystem::RecordInput, AppSystem::Update).chain());

        app.add_systems(Startup, spawn_camera);

        app.add_systems(Update, update_camera.run_if(in_state(Screen::Gameplay)));

        app.add_plugins((
            EntropyPlugin::<WyRand>::default(),
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: "bevy survivor".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
            EnemyPlugin,
            PlayerPlugin,
            MovementPlugin,
            ExperiencePlugin,
            HealthBarPlugin,
        ));

        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);

        app.add_plugins(screens::plugin);
    }
}

const PLAYER_DMG_STAT: f32 = 10.0;
const ENEMY_SIZE: f32 = 30.0;
const PLAYER_SIZE: f32 = 30.0;
const SPELL_SIZE: f32 = 16.0;
const XP_GAIN_GEM: i32 = 10;

/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

/// High-level groupings of systems for the app in the `Update` schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystem {
    /// Record player input.
    RecordInput,
    /// Do everything else
    Update,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}

/// Update the camera position by tracking the player.
fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
