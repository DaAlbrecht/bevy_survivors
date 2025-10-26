use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::gameplay::{PickUpSpell, spells::SpellType};

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::LevelUp), spawn_level_up_screen);
}

const NUMBER_OF_ITEM_CHOICES: usize = 3;

fn spawn_level_up_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let border_image = asset_server.load("kenny/panel-border-011.png");

    commands
        .spawn((
            Name::new("LevelUpRoot"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.012, 0.011, 0.011, 1.)),
            DespawnOnExit(Screen::LevelUp),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("LevelUp"),
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Percent(100.0),
                        height: Val::Percent(30.0),
                        column_gap: Val::Percent(10.0),
                        padding: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    for _ in 0..NUMBER_OF_ITEM_CHOICES {
                        let spell_index = rng.random_range(0..SpellType::ALL.len());

                        let spell_image: Handle<Image> = match SpellType::ALL[spell_index] {
                            SpellType::Scale => asset_server.load("Bullet.png"),
                            SpellType::Fireball => asset_server.load("Fireball_icon.png"),
                            SpellType::Lightning => asset_server.load("Lightning_icon.png"),
                            SpellType::Orb => asset_server.load("Orb_icon.png"),
                            SpellType::Thorn => asset_server.load("Thorn_base.png"),
                        };
                        parent
                            .spawn((
                                item_choice_widget(border_image.clone(), spell_image),
                                SpellType::ALL[spell_index],
                            ))
                            .observe(upgrade);
                    }
                });
        });
}

fn item_choice_widget(border_image: Handle<Image>, spell_image: Handle<Image>) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
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
        Button,
        Children::spawn(Spawn((
            Node {
                width: Val::Percent(80.),
                height: Val::Percent(80.),
                ..Default::default()
            },
            ImageNode::new(spell_image),
        ))),
    )
}

fn upgrade(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<Screen>>,
    spell_types: Query<&SpellType>,
) {
    let selected_spell = trigger.target();

    let pickup_event = PickUpSpell {
        spell_type: *spell_types
            .get(selected_spell)
            .expect("We should always find the SpellType the player chose"),
    };

    // Pickup spell
    commands.trigger(pickup_event);

    // Transition back to the gameplay
    next_state.set(Screen::Gameplay);
}
