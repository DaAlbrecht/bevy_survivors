use bevy::prelude::*;
use bevy::ui_widgets::observe;

use crate::gameplay::player::characters::Characters;
use crate::{screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::CharacterSelection), spawn_character_screen);
}

fn spawn_character_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = Characters::all().map(|c| c.get_spash_art(asset_server.clone()));
    commands.spawn((
        widget::ui_root("Character Selection  Screen"),
        DespawnOnExit(Screen::CharacterSelection),
        Children::spawn(SpawnIter(Characters::all().into_iter().zip(assets).map(
            |(character, asset)| (ImageNode::new(asset), Button, character, observe(select)),
        ))),
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
