use bevy::prelude::*;
use bevy::ui_widgets::observe;

use crate::gameplay::player::characters::Characters;
use crate::{screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::CharacterSelection), spawn_character_screen);
}

fn spawn_character_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Character Selection  Screen"),
        DespawnOnExit(Screen::CharacterSelection),
        children![
            (
                ImageNode::new(asset_server.load("player_wizard_.png")),
                Button,
                Characters::Wizzard,
                observe(select),
            ),
            (
                ImageNode::new(asset_server.load("player_knight_.png")),
                Button,
                Characters::Knight,
                observe(select),
            )
        ],
    ));
}

fn select(
    trigger: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    character: Query<&Characters>,
) {
    let selected_character = trigger.entity;
    if let Ok(character) = character.get(selected_character) {
        match character {
            Characters::Wizzard => info!("Wizzard"),
            Characters::Knight => info!("Knight"),
        };

        // Transition to the gameplay
        next_screen.set(Screen::Gameplay);
    }
}
