//! The level up menu.

use bevy::{color::palettes::basic, prelude::*, text::FontSmoothing};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    gameplay::{PickUpSpell, overlays::Overlay, spells::SpellType},
    theme::widget,
};

const NUMBER_OF_ITEM_CHOICES: usize = 3;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Overlay::LevelUp), spawn_level_up_menu);
}

fn spawn_level_up_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                        let spell_index = rng.random_range(0..SpellType::ALL.len());

                        let spell_image: Handle<Image> = match SpellType::ALL[spell_index] {
                            SpellType::Energy => asset_server.load("ui/icons/energy_item.png"),
                            SpellType::Circles => asset_server.load("ui/icons/circle_item.png"),
                            SpellType::Scale => asset_server.load("fx/scale.png"),
                            SpellType::Fireball => asset_server.load("ui/icons/fireball_item.png"),
                            SpellType::Lightning => {
                                asset_server.load("ui/icons/lightning_icon.png")
                            }
                            SpellType::Orb => asset_server.load("ui/icons/orbs_item.png"),
                            SpellType::Thorn => asset_server.load("fx/thorn_base.png"),
                            SpellType::Icelance => asset_server.load("ui/icons/icelance_item.png"),
                        };
                        parent
                            .spawn((
                                item_choice_widget(border_image.clone(), spell_image, &font),
                                SpellType::ALL[spell_index],
                            ))
                            .observe(upgrade);
                    }
                });
        });
}

fn item_choice_widget(
    border_image: Handle<Image>,
    spell_image: Handle<Image>,
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
                ImageNode::new(spell_image),
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
    //TODO: Get this from spell rarity proc
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
    spell_types: Query<&SpellType>,
) {
    let selected_spell = trigger.entity;

    let pickup_event = PickUpSpell {
        spell_type: *spell_types
            .get(selected_spell)
            .expect("We should always find the SpellType the player chose"),
    };

    // Pickup spell
    commands.trigger(pickup_event);

    // Transition back to the gameplay
    next_menu.set(Overlay::None);
}
