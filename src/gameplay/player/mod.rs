use avian2d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*, sprite_render::MeshMaterial2d};
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::{action::Action, actions};
use bevy_seedling::sample::AudioSample;

use crate::GameLayer;
use crate::gameplay::abilities::dash::Dash;
use crate::gameplay::abilities::heal::Heal;
use crate::gameplay::abilities::summon::Summon;
use crate::gameplay::abilities::{
    EAbility, QAbility, RAbility, UseEAbility, UseQAbility, UseRAbility,
};
use crate::gameplay::character_controller::CharacterController;
use crate::screens::Screen;
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
pub(crate) mod characters;
pub(crate) mod hit;
pub(crate) mod movement;

use animation::PlayerAnimation;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();

    app.add_plugins((animation::plugin, movement::plugin));

    app.load_resource::<PlayerAssets>();

    app.register_type::<XP>().register_type::<Level>();
    app.register_type::<Player>();

    app.add_systems(FixedUpdate, player_hit);

    app.add_observer(setup_player);
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
    pub sprite: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSample>>,
    #[dependency]
    pub shadow: Handle<Image>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sprite: assets.load("player_wizard_.png"),
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

/// This component will mark the player and be used to set the spawn in tiled
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub(crate) struct PlayerSpawnPoint;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(
    Health(100.),
    XpCollectionRange(150.0),
    XP(0.),
    Level(1.),
    CharacterController{speed: 100., ..default()},
    AccumulatedInput,
    DespawnOnExit::<Screen>(Screen::Gameplay),
)]
pub(crate) struct Player;

fn setup_player(
    add: On<Add, PlayerSpawnPoint>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    player_spawn_query: Query<&Transform, With<PlayerSpawnPoint>>,
    player_assets: If<Res<PlayerAssets>>,
    mut mesh: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2 { x: 64, y: 64 }, 11, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);

    let spawn_transform = *player_spawn_query.get(add.event().entity).unwrap();

    commands.spawn((
        Name::new("Player"),
        Player,
        player_input_actions(),
        PlayerAnimation::new(),
        LockedAxes::ROTATION_LOCKED,
        Collider::circle(16.),
        LinearDamping(10.0),
        Friction::ZERO,
        spawn_transform,
        Sprite::from_atlas_image(
            player_assets.sprite.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
        ),
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

    commands.trigger(crate::gameplay::PickUpWeapon {
        weapon_type: crate::gameplay::weapons::WeaponType::Fireball,
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
