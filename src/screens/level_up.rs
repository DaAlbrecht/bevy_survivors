use bevy::{
    prelude::*,
    ui::Val::{Percent, Px},
};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::gameplay::spells::SpellType;

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::LevelUp), spawn_level_up_screen);
}

const NUMBER_OF_ITEM_CHOICES: usize = 3;

fn spawn_level_up_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: GlobalEntropy<WyRand>,
) {
    let border_image = asset_server.load("kenny/panel-border-011.png");

    commands
        .spawn((
            Name::new("LevelUp"),
            Node {
                position_type: PositionType::Absolute,
                width: Percent(100.0),
                height: Percent(30.0),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Percent(10.0),
                padding: UiRect::new(Px(50.0), Px(50.0), Px(50.0), Px(50.0)),
                ..default()
            },
            StateScoped(Screen::LevelUp),
        ))
        .with_children(|parent| {
            for _ in 0..NUMBER_OF_ITEM_CHOICES {
                let spell_index = rng.gen_range(0..SpellType::ALL.len());

                let spell_image: Handle<Image> = match SpellType::ALL[spell_index] {
                    //TODO: use Scale icon
                    SpellType::Scale => asset_server.load("Fireball_icon.png"),
                    SpellType::Fireball => asset_server.load("Fireball_icon.png"),
                    SpellType::Lightning => asset_server.load("Lightning_icon.png"),
                    SpellType::Orb => asset_server.load("Orb_icon.png"),
                };
                parent
                    .spawn(item_choice_widget(border_image.clone(), spell_image))
                    .observe(upgrade);
            }
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

fn upgrade(_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<Screen>>) {
    info!("upgrade");
    next_state.set(Screen::Gameplay);
}
