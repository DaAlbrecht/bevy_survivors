use bevy::prelude::*;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use enemy::EnemyPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;

mod enemy;
mod level;
mod movement;
mod player;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (AppSet::RecordInput, AppSet::Update).chain());

        app.add_systems(Startup, spawn_camera);

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
            level::plugin,
        ));
    }
}

const PLAYER_DMG_STAT: f32 = 10.0;
const ENEMY_SIZE: f32 = 30.0;

/// High-level groupings of systems for the app in the `Update` schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSet {
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
