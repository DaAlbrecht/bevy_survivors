//! The pause menu.

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, text::FontSmoothing, ui::Val::*,
};

use crate::{
    gameplay::items::stats::{DerivedStats, StatId},
    menus::Menu,
    screens::Screen,
    theme::widget::{self, ButtonConfig},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stats_q: Single<&DerivedStats>,
) {
    let font: Handle<Font> = asset_server.load("ui/compass.ttf");
    let stats = stats_q.into_inner();

    commands.spawn((
        widget::ui_root("Pause Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Pause),
        children![
            widget::header("Game Paused"),
            stats_display(&font, stats),
            pause_actions()
        ],
    ));
}

fn pause_actions() -> impl Bundle {
    (
        Name::new("Pause Actions"),
        Node {
            column_gap: px(20),
            ..default()
        },
        children![
            widget::button(
                "Quit to title",
                quit_to_title,
                ButtonConfig {
                    width: Px(200.0),
                    height: Px(40.0),
                    text_size: 18.0,
                }
            ),
            widget::button(
                "Settings",
                open_settings_menu,
                ButtonConfig {
                    width: Px(200.0),
                    height: Px(40.0),
                    text_size: 18.0,
                }
            ),
            widget::button(
                "Continue",
                close_menu,
                ButtonConfig {
                    width: Px(200.0),
                    height: Px(40.0),
                    text_size: 18.0,
                }
            ),
        ],
    )
}

fn stats_display(font: &Handle<Font>, stats_q: &DerivedStats) -> impl Bundle {
    let stats = stats_q.0;

    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(8)),
            ..default()
        },
        Children::spawn((
            Spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                Children::spawn((
                    Spawn(section_header("Combat Stats", font)),
                    Spawn(stat_row(stats, StatId::Attack, font)),
                    Spawn(stat_row(stats, StatId::CritChance, font)),
                    Spawn(stat_row(stats, StatId::CritDamage, font)),
                    Spawn(stat_row(stats, StatId::AttackSpeed, font)),
                )),
            )),
            Spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                Children::spawn((
                    Spawn(section_header("Movement & Survivability", font)),
                    Spawn(stat_row(stats, StatId::MoveSpeed, font)),
                    Spawn(stat_row(stats, StatId::MaxHealth, font)),
                    Spawn(stat_row(stats, StatId::Armor, font)),
                    Spawn(stat_row(stats, StatId::Recovery, font)),
                )),
            )),
            Spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                Children::spawn((
                    Spawn(section_header("Weapon Modifiers", font)),
                    Spawn(stat_row(stats, StatId::ProjectileCount, font)),
                    Spawn(stat_row(stats, StatId::Duration, font)),
                    Spawn(stat_row(stats, StatId::Area, font)),
                    Spawn(stat_row(stats, StatId::Cooldown, font)),
                )),
            )),
            Spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                Children::spawn((
                    Spawn(section_header("Utility", font)),
                    Spawn(stat_row(stats, StatId::PickupRange, font)),
                )),
            )),
        )),
    )
}

fn section_header(title: &str, font: &Handle<Font>) -> impl Bundle {
    (
        Text::new(title),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.3)),
    )
}

fn stat_row(
    stats: crate::gameplay::items::stats::Stats,
    stat_id: StatId,
    font: &Handle<Font>,
) -> impl Bundle {
    let label = stat_id.display_name();
    let formatted_value = stats.format(stat_id);

    (
        Text::new(format!("{label}: {formatted_value}")),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
    )
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
