use avian2d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*, sprite_render::MeshMaterial2d};
use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::{action::Action, actions};
use bevy_seedling::sample::AudioSample;

use crate::GameLayer;
use crate::gameplay::character_controller::CharacterController;
use crate::{
    asset_tracking::LoadResource,
    gameplay::{
        Health,
        healthbar::HealthBarMaterial,
        player::{
            hit::player_hit,
            movement::{AccumulatedInput, Move},
        },
    },
};

pub(crate) mod animation;
pub(crate) mod hit;
pub(crate) mod movement;
use animation::PlayerAnimation;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();

    app.add_plugins((animation::plugin, movement::plugin));

    app.load_resource::<PlayerAssets>();

    app.register_ldtk_entity::<PlayerBundle>("Player");
    app.register_type::<XP>().register_type::<Level>();

    app.add_observer(player_hit);
    app.add_observer(setup_player);
}

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Event, Reflect)]
pub(crate) struct PlayerHitEvent {
    pub dmg: f32,
}

#[derive(Component, Reflect)]
pub(crate) struct Direction(pub Vec3);

#[derive(Component, Reflect)]
pub(crate) struct XpCollectionRange(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct XP(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Level(pub f32);

#[derive(Component)]
#[relationship_target(relationship = AddToInventory)]
#[derive(Reflect)]
pub(crate) struct Inventory(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Inventory)]
#[derive(Reflect)]
pub(crate) struct AddToInventory(pub Entity);

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub steps: Vec<Handle<AudioSample>>,
    #[dependency]
    pub shadow: Handle<Image>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            steps: vec![
                assets.load("audio/sound_effects/stone_run_1.ogg"),
                assets.load("audio/sound_effects/stone_run_2.ogg"),
                assets.load("audio/sound_effects/stone_run_3.ogg"),
                assets.load("audio/sound_effects/stone_run_4.ogg"),
                assets.load("audio/sound_effects/stone_run_5.ogg"),
            ],
            shadow: assets.load("fx/shadow.png"),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(
    Health(100.),
    XpCollectionRange(150.0),
    XP(0.),
    Level(1.),
    CharacterController{speed: 100.},
    AccumulatedInput,
)]
pub(crate) struct Player;

fn setup_player(
    add: On<Add, Player>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    player_assets: If<Res<PlayerAssets>>,
    mut mesh: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    commands.entity(add.entity).insert((
        player_input_actions(),
        // A texture atlas is a way to split a single image into a grid of related images.
        // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
        PlayerAnimation::new(),
        LockedAxes::ROTATION_LOCKED,
        // Prevent the player from getting impacted by external forces.
        RigidBody::Kinematic,
        Collider::rectangle(32., 32.),
        CollisionLayers::new(
            GameLayer::Player,
            [
                GameLayer::Enemy,
                GameLayer::Default,
                GameLayer::EnemyProjectiles,
            ],
        ),
        children![
            (
                Mesh2d(mesh.add(Rectangle::new(32.0, 5.0))),
                MeshMaterial2d(health_bar_materials.add(HealthBarMaterial {
                    foreground_color: tailwind::GREEN_300.into(),
                    background_color: tailwind::RED_300.into(),
                    percent: 1.,
                })),
                Transform::from_xyz(0.0, -25.0, 0.),
            ),
            (
                Sprite {
                    image: player_assets.shadow.clone(),

                    ..Default::default()
                },
                Transform::from_xyz(-4., -16.0, -0.1).with_scale(Vec3 {
                    x: 2.,
                    y: 1.,
                    z: 1.
                })
            )
        ],
    ));

    commands.trigger(crate::gameplay::PickUpSpell {
        spell_type: crate::gameplay::spells::SpellType::Orb,
    });
}

fn player_input_actions() -> impl Bundle {
    actions!(Player[
        (
            Action::<Move>::new(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick()
            )),
        ),
    ])
}
