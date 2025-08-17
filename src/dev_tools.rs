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

use crate::{
    gameplay::{
        PickUpSpell,
        attacks::{Spell, SpellType},
        experience::LevelUpEvent,
    },
    screens::Screen,
};

const TOGGLE_DEBUG_UI_KEY: KeyCode = KeyCode::Backquote;
const TRIGGER_LEVEL_UP_KEY: KeyCode = KeyCode::F1;
const TOGGLE_INSEPCTOR: KeyCode = KeyCode::F2;
const ADD_ALL_SPELLS: KeyCode = KeyCode::F3;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            toggle_debug_ui.run_if(input_just_pressed(TOGGLE_DEBUG_UI_KEY)),
            trigger_level_up.run_if(input_just_pressed(TRIGGER_LEVEL_UP_KEY)),
            add_all_spells.run_if(input_just_pressed(ADD_ALL_SPELLS)),
        ),
    );

    app.add_plugins((
        EguiPlugin::default(),
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

fn all_spells() -> &'static [SpellType] {
    //This is kinda annoying since we have to remember to add each new spell..
    &[
        SpellType::Scale,
        SpellType::Fireball,
        SpellType::Orb,
        SpellType::Lightning,
    ]
}

fn add_all_spells(mut commands: Commands, owned_spells: Query<&SpellType, With<Spell>>) {
    let owned_spells = owned_spells.iter().copied().collect::<Vec<SpellType>>();
    for spell in all_spells() {
        if !owned_spells.contains(spell) {
            commands.trigger(PickUpSpell { spell_type: *spell });
        }
    }
}
