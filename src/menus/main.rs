//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    AssetStates,
    menus::Menu,
    screens::Screen,
    theme::{
        palette::SCREEN_BACKGROUND,
        widget::{self, ButtonConfig},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        BackgroundColor(SCREEN_BACKGROUND),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
        children![
            (
                Name::new("Splash image"),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode::new(asset_server.load("splash_bs.png",)),
            ),
            widget::button("Play", start, ButtonConfig::default()),
            widget::button("Settings", open_settings_menu, ButtonConfig::default()),
            widget::button("Exit", exit_app, ButtonConfig::default()),
        ],
    ));
}

fn start(
    _: On<Pointer<Click>>,
    asset_state: Res<State<AssetStates>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    match asset_state.get() {
        AssetStates::AssetLoading => next_screen.set(Screen::Loading),
        AssetStates::Next => next_menu.set(Menu::CharacterSelection),
    }
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
