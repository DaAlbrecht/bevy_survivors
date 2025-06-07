use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
    ui::UiDebugOptions,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{StateInspectorPlugin, WorldInspectorPlugin},
};

use crate::{gameplay::experience::LevelUpEvent, screens::Screen};

const TOGGLE_DEBUG_UI_KEY: KeyCode = KeyCode::Backquote;
const TRIGGER_LEVEL_UP_KEY: KeyCode = KeyCode::F1;
const TOGGLE_INSEPCTOR: KeyCode = KeyCode::F2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            toggle_debug_ui.run_if(input_just_pressed(TOGGLE_DEBUG_UI_KEY)),
            trigger_level_up.run_if(input_just_pressed(TRIGGER_LEVEL_UP_KEY)),
        ),
    );

    app.add_plugins(EguiPlugin {
        enable_multipass_for_primary_context: true,
    })
    .add_plugins((
        WorldInspectorPlugin::new().run_if(input_toggle_active(true, TOGGLE_INSEPCTOR)),
        StateInspectorPlugin::<Screen>::new().run_if(input_toggle_active(true, TOGGLE_INSEPCTOR)),
    ));

    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextFont {
                font_size: 42.0,
                ..default()
            },
            refresh_interval: core::time::Duration::from_millis(100),
            enabled: true,
            ..default()
        },
    });
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn trigger_level_up(mut commands: Commands) {
    commands.trigger(LevelUpEvent);
}
