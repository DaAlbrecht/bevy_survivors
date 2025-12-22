use bevy::{prelude::*, ui::Val::*, ui_widgets::observe};

use crate::gameplay::player::characters::Characters;
use crate::theme::palette::{
    BUTTON_BACKGROUND, BUTTON_HOVERED_BACKGROUND, BUTTON_PRESSED_BACKGROUND,
};
use crate::theme::prelude::InteractionPalette;
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
            |(character, asset)| {
                (
                    Node {
                        width: Percent(100.0),
                        height: Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    children![
                        (
                            Node {
                                width: Percent(100.0),
                                height: Percent(80.0),
                                justify_items: JustifyItems::Center,
                                ..default()
                            },
                            ImageNode::new(asset),
                        ),
                        (
                            Node {
                                width: Percent(100.0),
                                height: Percent(20.0),
                                justify_items: JustifyItems::Center,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            children![(
                                Node {
                                    padding: UiRect {
                                        left: Val::Px(20.),
                                        right: Val::Px(20.),
                                        top: Val::Px(0.),
                                        bottom: Val::Px(0.)
                                    },
                                    ..default()
                                },
                                TextLayout::new_with_justify(Justify::Center),
                                character,
                                widget::label("Select"),
                                BorderRadius::MAX,
                                BackgroundColor(BUTTON_BACKGROUND.into()),
                                InteractionPalette {
                                    none: BUTTON_BACKGROUND.into(),
                                    hovered: BUTTON_HOVERED_BACKGROUND.into(),
                                    pressed: BUTTON_PRESSED_BACKGROUND.into(),
                                },
                                Button,
                                observe(select),
                            )],
                        )
                    ],
                )
            },
        ))),
    ));
}

fn select(
    trigger: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    character: Query<&Characters>,
) {
    info!("clicky");
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
