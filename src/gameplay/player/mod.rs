use crate::gameplay::weapons::prelude::*;
use avian2d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*, sprite_render::MeshMaterial2d};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::{action::Action, actions};
use bevy_seedling::sample::AudioSample;

use crate::gameplay::abilities::prelude::*;
use crate::gameplay::character_controller::CharacterController;
use crate::gameplay::{
    Health,
    healthbar::HealthBarMaterial,
    player::{
        hit::player_hit,
        movement::{AccumulatedInput, Move},
    },
};
use crate::{AssetStates, GameLayer};
pub(crate) mod animation;
pub(crate) mod hit;
pub(crate) mod movement;
use animation::PlayerAnimation;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();
    app.configure_loading_state(
        LoadingStateConfig::new(AssetStates::AssetLoading).load_collection::<PlayerAssets>(),
    );

    app.add_plugins((animation::plugin, movement::plugin));

    app.register_ldtk_entity::<PlayerBundle>("Player");
    app.register_type::<XP>().register_type::<Level>();

    app.add_systems(FixedUpdate, player_hit);

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

#[derive(Event)]
pub struct PlayerSetupComplete;

#[derive(Event, Reflect)]
pub(crate) struct PlayerHitEvent {
    pub dmg: f32,
}

#[derive(Component, Reflect, Default)]
pub(crate) struct Direction(pub Vec3);

#[derive(Component, Reflect)]
pub(crate) struct LastFacingDirection(pub Vec2);

impl Default for LastFacingDirection {
    fn default() -> Self {
        Self(Vec2::X)
    }
}

#[derive(Component, Reflect)]
pub(crate) struct XpCollectionRange(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct XP(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Level(pub f32);

#[derive(Component)]
#[relationship_target(relationship = InInventoryOf)]
#[derive(Reflect)]
pub(crate) struct Inventory(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Inventory)]
#[derive(Reflect)]
pub(crate) struct InInventoryOf(pub Entity);

#[derive(AssetCollection, Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[asset(
        paths(
            "audio/sound_effects/stone_run_1.ogg",
            "audio/sound_effects/stone_run_2.ogg",
            "audio/sound_effects/stone_run_3.ogg",
            "audio/sound_effects/stone_run_4.ogg",
            "audio/sound_effects/stone_run_5.ogg",
        ),
        collection(typed)
    )]
    pub steps: Vec<Handle<AudioSample>>,
    #[asset(path = "fx/shadow.png")]
    pub shadow: Handle<Image>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(
    Health(100.),
    XpCollectionRange(150.0),
    XP(0.),
    Level(1.),
    CharacterController{speed: 100., ..default()},
    AccumulatedInput,
    LastFacingDirection,
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
        PlayerAnimation::new(),
        LockedAxes::ROTATION_LOCKED,
        // Prevent the player from getting impacted by external forces.
        RigidBody::Kinematic,
        Collider::circle(16.),
        Friction::ZERO,
        CollisionLayers::new(
            GameLayer::Player,
            [
                GameLayer::Enemy,
                GameLayer::Default,
                GameLayer::EnemyProjectiles,
            ],
        ),
        CollisionEventsEnabled,
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
        CollidingEntities::default(),
    ));

    commands.trigger(PickUpWeaponEvent {
        kind: WeaponKind::Circles,
    });

    commands.spawn((QAbility, Heal));
    commands.spawn((EAbility, Dash));
    commands.spawn((RAbility, Summon));

    commands.trigger(PlayerSetupComplete);
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
        (
            Action::<UseQAbility>::new(),
            bindings![KeyCode::KeyQ]
        ),
        (
            Action::<UseEAbility>::new(),
            bindings![KeyCode::KeyE]
        ),
        (
            Action::<UseRAbility>::new(),
            bindings![KeyCode::KeyR]
        )
    ])
}
