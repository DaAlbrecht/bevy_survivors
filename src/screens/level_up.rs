use bevy::{
    color::palettes::css::YELLOW,
    prelude::*,
    ui::Val::{Percent, Px},
};

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::LevelUp), spawn_level_up_screen);
}

const NUMBER_OF_ITEM_CHOICES: usize = 3;

fn spawn_level_up_screen(mut commands: Commands) {
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
                parent.spawn(item_choice_widget()).observe(upgrade);
            }
        });
}

fn item_choice_widget() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(YELLOW.into()),
        Button,
    )
}

fn upgrade(_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<Screen>>) {
    info!("upgrade");
    next_state.set(Screen::Gameplay);
}
