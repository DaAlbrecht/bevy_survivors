//! The level up menu.

use bevy::{prelude::*, window::WindowResized};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud);
    app.add_systems(Update, on_resize_system);
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let hud_image = asset_server.load("ui/hud.png");
    let rain_image = asset_server.load("ui/icons/rain_spell.png");
    let fireball_image = asset_server.load("ui/icons/fireball_item.png");
    // let orb_image = asset_server.load("ui/orbs_item.png");

    commands
        .spawn((
            (
                Name::new("HudRoot"),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect {
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                // Don't block picking events for other UI roots.
                Pickable::IGNORE,
            ),
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("LevelUp"),
                    Node {
                        width: Val::Px(352.),
                        height: Val::Px(96.),
                        display: Display::Flex,
                        position_type: PositionType::Relative,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("BG"),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ImageNode {
                            image: hud_image,
                            image_mode: NodeImageMode::Auto,
                            ..default()
                        },
                    ));
                    parent
                        .spawn((
                            Name::new("Grid"),
                            Node {
                                display: Display::Grid,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                grid_template_columns: RepeatedGridTrack::flex(11, 1.0),
                                grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            item_rect(parent, 2, 1, 2, 2, rain_image);
                            item_rect(parent, 3, 10, 1, 1, fireball_image);
                            // item_rect(parent, 2, 8, 1, 1, orb_image);
                        });
                });
        });
}

fn item_rect(
    builder: &mut ChildSpawnerCommands,
    row_start: i16,
    col_start: i16,
    col_span: u16,
    row_span: u16,
    spell_image: Handle<Image>,
) {
    builder.spawn((
        Node {
            display: Display::Grid,
            grid_column: GridPlacement::start(col_start).set_span(col_span),
            grid_row: GridPlacement::start(row_start).set_span(row_span),
            padding: UiRect::all(px(3)),
            ..default()
        },
        ImageNode {
            image: spell_image,
            image_mode: NodeImageMode::Auto,
            ..default()
        },
    ));
}

/// This system shows how to respond to a window being resized.
/// Whenever the window is resized, the text will update with the new resolution.
fn on_resize_system(mut resize_reader: MessageReader<WindowResized>) {
    for e in resize_reader.read() {
        // When resolution is being changed
        info!("Window resized to: {} x {}", e.width, e.height);
    }
}
