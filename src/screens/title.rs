//! The title screen that appears when the game starts.

use bevy::prelude::*;

use crate::widgets;

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen);
}

fn spawn_title_screen(mut commands: Commands) {
    commands.spawn((
        widgets::ui_root("Title Screen"),
        StateScoped(Screen::Title),
        #[cfg(not(target_family = "wasm"))]
        children![
            widgets::button("Play", enter_loading_or_gameplay_screen),
            widgets::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![widgets::button("Play", enter_loading_or_gameplay_screen),],
    ));
}

fn enter_loading_or_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    next_screen.set(Screen::Gameplay);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
