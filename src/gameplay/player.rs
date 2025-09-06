use bevy_enhanced_input::prelude::*;

use bevy::{color::palettes::css, prelude::*};
use bevy_enhanced_input::action::Action;
use bevy_enhanced_input::actions;

use super::healthbar::HealthBarMaterial;
use crate::{AppSystems, gameplay::Health, screens::Screen};

pub(crate) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();

    app.add_systems(Startup, spawn_player);

    app.add_systems(OnEnter(Screen::Gameplay), show_player);

    app.add_systems(
        Update,
        (move_player.in_set(AppSystems::RecordInput)).run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(player_hit);

    app.register_type::<XP>().register_type::<Level>();
}

#[derive(Component)]
#[require(
    Health(100.),
    Transform::from_xyz(50., 0., 0.),
    XpCollectionRange(150.0),
    XP(0.),
    Level(1.)
)]
#[derive(Reflect)]
pub(crate) struct Player;

#[derive(Event)]
#[derive(Reflect)]
pub(crate) struct PlayerHitEvent {
    pub dmg: f32,
}

#[derive(Component)]
#[derive(Reflect)]
pub(crate) struct Direction(pub Vec3);

#[derive(Component)]
#[derive(Reflect)]
pub(crate) struct XpCollectionRange(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct XP(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Level(pub f32);

#[derive(InputAction)]
#[action_output(Vec2)]
pub(crate) struct Move;

#[derive(Component)]
#[relationship_target(relationship = AddToInventory)]
#[derive(Reflect)]
pub(crate) struct Inventory(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Inventory)]
#[derive(Reflect)]
pub(crate) struct AddToInventory(pub Entity);

pub(crate) fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    commands
        .spawn((
            Player,
            Name::new("Player"),
            Sprite::from_image(asset_server.load("Player.png")),
            Transform::from_xyz(50., 0., 0.),
            actions!(Player[
                (
                    Action::<Move>::new(),
                    SmoothNudge::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick()
                    )),
                    Scale::splat(100.),
                ),
            ]),
            Visibility::Hidden,
        ))
        .with_child((
            Mesh2d(mesh.add(Rectangle::new(32.0, 5.0))),
            MeshMaterial2d(health_bar_materials.add(HealthBarMaterial {
                foreground_color: css::GREEN.into(),
                background_color: css::RED.into(),
                percent: 1.,
            })),
            Transform::from_xyz(0.0, -25.0, 0.),
        ));

    //Default player has Scale attack
    commands.trigger(crate::gameplay::PickUpSpell {
        spell_type: crate::gameplay::spells::SpellType::Thorn,
    });
}

fn player_hit(
    trigger: Trigger<PlayerHitEvent>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    mut player_q: Query<&mut Health, With<Player>>,
    healthbar_material_q: Query<&MeshMaterial2d<HealthBarMaterial>>,
) -> Result {
    let mut health = player_q.single_mut()?;
    health.0 -= trigger.dmg;
    debug!("attacking player, player_health: {}", health.0);
    let per = health.0 / 100.;

    let handle = healthbar_material_q.single()?.clone_weak();
    let material = health_bar_materials.get_mut(&handle).unwrap();
    material.percent = per;

    Ok(())
}

fn move_player(
    move_action: Single<&Action<Move>>,
    mut player_transform_q: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) -> Result {
    let velocity = move_action.extend(0.0);
    let mut player_transform = player_transform_q.single_mut()?;
    player_transform.translation += velocity * time.delta_secs();

    Ok(())
}

fn show_player(mut visibility_q: Query<&mut Visibility, With<Player>>) -> Result {
    let mut visibility = visibility_q.single_mut()?;
    *visibility = Visibility::Visible;
    Ok(())
}
