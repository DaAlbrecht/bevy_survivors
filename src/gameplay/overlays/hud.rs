use bevy::prelude::*;

use crate::gameplay::abilities;
use crate::{gameplay::player::PlayerSetupComplete, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_hud);
    app.add_systems(
        Update,
        update_ability_cooldowns.run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component)]
struct AbilityCooldownBar;

#[derive(Component)]
struct AbilityIconUI;

#[derive(Component)]
struct TracksAbility(Entity);

enum AbilitySlot {
    Q { entity: Entity, icon: Handle<Image> },
    E { entity: Entity, icon: Handle<Image> },
    R { entity: Entity, icon: Handle<Image> },
}

fn spawn_hud(
    _trigger: On<PlayerSetupComplete>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    abilities: Query<(
        Entity,
        &abilities::AbilityAssets,
        Option<&abilities::QAbility>,
        Option<&abilities::EAbility>,
        Option<&abilities::RAbility>,
    )>,
) {
    let hud_image = asset_server.load("ui/hud.png");
    //TODO: Dynamic Spells
    let fireball_image = asset_server.load("ui/icons/fireball_item.png");

    let mut ability_slots = Vec::new();
    for (entity, ability_assets, q, e, r) in &abilities {
        let slot = if q.is_some() {
            AbilitySlot::Q {
                entity,
                icon: ability_assets.icon.clone(),
            }
        } else if e.is_some() {
            AbilitySlot::E {
                entity,
                icon: ability_assets.icon.clone(),
            }
        } else if r.is_some() {
            AbilitySlot::R {
                entity,
                icon: ability_assets.icon.clone(),
            }
        } else {
            continue;
        };

        ability_slots.push(slot);
    }

    commands
        .spawn((
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
            Pickable::IGNORE,
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("LevelUp"),
                    Node {
                        width: Val::Px(416.),
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
                                grid_template_columns: RepeatedGridTrack::flex(13, 1.0),
                                grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            for slot in ability_slots {
                                let (ability_entity, ability_icon, col_start) = match slot {
                                    AbilitySlot::Q { entity, icon } => (entity, icon, 1),
                                    AbilitySlot::E { entity, icon } => (entity, icon, 4),
                                    AbilitySlot::R { entity, icon } => (entity, icon, 7),
                                };

                                parent
                                    .spawn((
                                        item_rect(2, col_start, 2, 2, ability_icon),
                                        AbilityIconUI,
                                        TracksAbility(ability_entity),
                                    ))
                                    .with_children(|p| {
                                        p.spawn((
                                            AbilityCooldownBar,
                                            TracksAbility(ability_entity),
                                            cooldown_bar(),
                                        ));
                                    });
                            }
                            parent.spawn(item_rect(3, 10, 1, 1, fireball_image));
                        });
                });
        });
}

fn cooldown_bar() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(0.0),
            height: Val::Percent(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
    )
}

fn item_rect(
    row_start: i16,
    col_start: i16,
    col_span: u16,
    row_span: u16,
    slot_image: Handle<Image>,
) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            grid_column: GridPlacement::start(col_start).set_span(col_span),
            grid_row: GridPlacement::start(row_start).set_span(row_span),
            padding: UiRect::all(Val::Px(3.)),
            position_type: PositionType::Relative,
            ..default()
        },
        ImageNode {
            image: slot_image,
            image_mode: NodeImageMode::Auto,
            ..default()
        },
    )
}

fn update_ability_cooldowns(
    abilities: Query<(Entity, &abilities::AbilityCooldown)>,
    mut bars: Query<(&TracksAbility, &mut Node), With<AbilityCooldownBar>>,
    mut icons: Query<(&TracksAbility, &mut ImageNode), With<AbilityIconUI>>,
) {
    for (ability_entity, cooldown) in &abilities {
        let is_ready = cooldown.0.is_finished();
        let percent_remaining = if is_ready {
            0.0
        } else {
            1.0 - cooldown.0.fraction()
        };

        for (tracks, mut node) in &mut bars {
            if tracks.0 == ability_entity {
                node.width = Val::Percent(percent_remaining * 100.0);
            }
        }

        for (tracks, mut image_node) in &mut icons {
            if tracks.0 == ability_entity {
                image_node.color = if is_ready {
                    Color::WHITE
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
                };
            }
        }
    }
}
