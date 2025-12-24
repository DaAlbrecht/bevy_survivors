use bevy::text::FontSmoothing;
use bevy::{prelude::*, ui::Val::*, ui_widgets::observe};

use crate::gameplay::player::Player;
use crate::gameplay::player::characters::Characters;
use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::menus::Menu;
use crate::screens::Screen;
use crate::theme::palette::{
    BUTTON_BACKGROUND, BUTTON_HOVERED_BACKGROUND, BUTTON_PRESSED_BACKGROUND,
};
use crate::theme::prelude::InteractionPalette;
use crate::theme::widget;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::CharacterSelection), spawn_character_screen);
}

fn spawn_character_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let font: Handle<Font> = asset_server.load("ui/compass.ttf");

    commands.spawn((
        widget::ui_root("Character Selection Screen"),
        DespawnOnExit(Menu::CharacterSelection),
        Children::spawn(Spawn((
            Name::new("Character Selection Container"),
            Node {
                width: Percent(100.0),
                height: Percent(100.0),
                padding: UiRect::all(Px(40.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Px(20.0),
                ..default()
            },
            Children::spawn((
                Spawn((
                    Text::new("Select Your Character"),
                    TextFont {
                        font: font.clone(),
                        font_size: 32.0,
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    },
                )),
                Spawn(spawn_character_grid(asset_server, texture_atlases)),
                Spawn(widget::button("Back", back)),
            )),
        ))),
    ));
}

fn back(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn spawn_character_grid(
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) -> impl Bundle {
    (
        Node {
            width: Percent(100.0),
            height: Percent(100.0),
            display: Display::Grid,
            flex_grow: 1.0,
            grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
            column_gap: Px(20.0),
            row_gap: Px(20.0),
            ..default()
        },
        spawn_character_cards(asset_server, texture_atlases),
    )
}

fn spawn_character_cards(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) -> impl Bundle {
    let assets: Vec<_> = Characters::all()
        .into_iter()
        .map(|c| {
            let texture = c.get_spash_art(asset_server.clone());
            let layout = c.get_texture_atlas();
            let atlas_handle = texture_atlases.add(layout);
            let idle = c.get_idle_indicies();

            (c, texture, atlas_handle, idle)
        })
        .collect();

    Children::spawn(SpawnIter(assets.into_iter().map(
        |(character, texture, atlas, idle)| {
            (
                Node {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Children::spawn((
                    Spawn((
                        Node {
                            width: Px(128.0),
                            height: Px(128.0),
                            ..default()
                        },
                        ImageNode::from_atlas_image(
                            texture,
                            TextureAtlas {
                                layout: atlas,
                                index: idle.0,
                            },
                        ),
                        AnimationIndices {
                            first: idle.0,
                            last: idle.1,
                        },
                        AnimationTimer {
                            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                        },
                        Pickable::default(),
                    )),
                    Spawn((
                        Node {
                            padding: UiRect {
                                left: Val::Px(20.),
                                right: Val::Px(20.),
                                top: Val::Px(0.),
                                bottom: Val::Px(0.),
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
                    )),
                )),
            )
        },
    )))
}

fn select(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    mut next_screen: ResMut<NextState<Screen>>,
    character: Query<&Characters>,
) {
    let selected_character = trigger.entity;
    if let Ok(character) = character.get(selected_character) {
        commands.spawn((Player, *character));

        // Transition to the gameplay
        next_screen.set(Screen::Gameplay);
    }
}
