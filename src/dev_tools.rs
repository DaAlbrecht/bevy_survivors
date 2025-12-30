use avian2d::prelude::PhysicsGizmos;
#[cfg(not(target_family = "wasm"))]
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};

use bevy::{
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{StateInspectorPlugin, WorldInspectorPlugin},
};

use crate::{gameplay::overlays::experience::LevelUpEvent, screens::Screen};

const TOGGLE_DEBUG_UI_KEY: KeyCode = KeyCode::Backquote;
const TRIGGER_LEVEL_UP_KEY: KeyCode = KeyCode::F1;
const TOGGLE_INSEPCTOR: KeyCode = KeyCode::F2;
// const ADD_ALL_WEAPONS: KeyCode = KeyCode::F3;
const TOGGLE_COLLIDERS_KEY: KeyCode = KeyCode::F4;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            toggle_debug_ui.run_if(input_just_pressed(TOGGLE_DEBUG_UI_KEY)),
            toggle_colliders.run_if(input_just_pressed(TOGGLE_COLLIDERS_KEY)),
            trigger_level_up.run_if(input_just_pressed(TRIGGER_LEVEL_UP_KEY)),
        ),
    );

    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::new().run_if(input_toggle_active(true, TOGGLE_INSEPCTOR)),
        StateInspectorPlugin::<Screen>::new().run_if(input_toggle_active(true, TOGGLE_INSEPCTOR)),
    ));

    #[cfg(not(target_family = "wasm"))]
    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextFont {
                font_size: 42.0,
                ..default()
            },
            refresh_interval: core::time::Duration::from_millis(100),
            enabled: true,
            frame_time_graph_config: {
                FrameTimeGraphConfig {
                    enabled: true,
                    min_fps: 60.,
                    target_fps: 120.,
                }
            },
            ..default()
        },
    });
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn toggle_colliders(mut gizmo_config_store: ResMut<GizmoConfigStore>) {
    let gizmo_config: &mut GizmoConfig = gizmo_config_store.config_mut::<PhysicsGizmos>().0;
    gizmo_config.enabled = !gizmo_config.enabled;
}

fn trigger_level_up(mut commands: Commands) {
    commands.trigger(LevelUpEvent);
}
