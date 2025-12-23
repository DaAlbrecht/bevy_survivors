//! The level up menu.
use bevy::{color::palettes::basic, prelude::*, text::FontSmoothing};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    gameplay::{
        overlays::Overlay,
        ws::{
            assets::WeaponMap,
            prelude::{PickUpWeaponEvent, WeaponKind},
        },
    },
    theme::widget,
};

const NUMBER_OF_ITEM_CHOICES: usize = 3;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Overlay::LevelUp), spawn_level_up_menu);
}

fn spawn_level_up_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weapons: Res<WeaponMap>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let border_image = asset_server.load("kenny/panel-border-011.png");
    let font: Handle<Font> = asset_server.load("ui/compass.ttf");

    commands
        .spawn((
            widget::ui_root("LevelUpRoot"),
            DespawnOnExit(Overlay::LevelUp),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("LevelUp"),
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        column_gap: Val::Percent(10.0),
                        padding: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    for _ in 0..NUMBER_OF_ITEM_CHOICES {
                        let weapon_index = rng.random_range(0..WeaponKind::ALL.len());
                        let kind = WeaponKind::ALL[weapon_index];
                        let spec = weapons.get(&kind).expect("expect spec for kind");

                        let icon = spec.icon.clone();

                        parent
                            .spawn((
                                item_choice_widget(border_image.clone(), icon, &font),
                                WeaponKind::ALL[weapon_index],
                            ))
                            .observe(upgrade);
                    }
                });
        });
}

fn item_choice_widget(
    border_image: Handle<Image>,
    weapon_image: Handle<Image>,
    font: &Handle<Font>,
) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: Val::Px(40.0),
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        ImageNode {
            image: border_image,
            image_mode: NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(22.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
            ..default()
        },
        //Dark semi-transparent
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
        Button,
        Children::spawn((
            Spawn((
                Node {
                    width: Val::Px(128.),
                    height: Val::Px(128.),
                    ..Default::default()
                },
                ImageNode::new(weapon_image),
            )),
            Spawn(item_desc(font)),
            Spawn(item_txt(font)),
        )),
    )
}

fn item_desc(font: &Handle<Font>) -> impl Bundle {
    (
        Node {
            width: Val::Px(100.),
            height: Val::Percent(100.),
            flex_grow: 1.0,
            ..Default::default()
        },
        Children::spawn((Spawn((
            Text::new("Damage Increase"),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextLayout::new_with_justify(Justify::Center),
            TextColor(Color::WHITE),
        )),)),
    )
}

fn item_txt(font: &Handle<Font>) -> impl Bundle {
    //TODO: Get this from weapon rarity proc
    let colors = [basic::WHITE, basic::GREEN, basic::BLUE, basic::RED];
    let rng = &mut rand::rng();
    let color = colors[rng.random_range(0..colors.len())];
    (
        Node {
            width: Val::Px(100.),
            height: Val::Percent(100.),
            ..Default::default()
        },
        Children::spawn((Spawn((
            Text::new("Upgrade"),
            TextFont {
                font: font.clone(),
                font_size: 32.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextLayout::new_with_justify(Justify::Center),
            TextColor(color.into()),
        )),)),
    )
}

fn upgrade(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    mut next_menu: ResMut<NextState<Overlay>>,
    weapon_types: Query<&WeaponKind>,
) {
    let selected_weapon = trigger.entity;

    let pickup_event = PickUpWeaponEvent {
        kind: *weapon_types
            .get(selected_weapon)
            .expect("We should always find the WeaponType the player chose"),
    };

    // Pickup weapon
    commands.trigger(pickup_event);

    // Transition back to the gameplay
    next_menu.set(Overlay::None);
}
