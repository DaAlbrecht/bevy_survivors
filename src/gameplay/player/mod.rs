use crate::gameplay::weapons::kind::WeaponKind;
use crate::gameplay::weapons::systems::pickup::PickUpWeaponEvent;
use crate::screens::Screen;
use avian2d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*, sprite_render::MeshMaterial2d};
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input::{action::Action, actions};
use bevy_seedling::sample::AudioSample;

use crate::gameplay::abilities;
use crate::gameplay::character_controller::CharacterController;
use crate::gameplay::player::characters::Characters;
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
pub(crate) mod characters;
pub(crate) mod hit;
pub(crate) mod movement;

use animation::PlayerAnimation;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();
    app.configure_loading_state(
        LoadingStateConfig::new(AssetStates::AssetLoading).load_collection::<PlayerAssets>(),
    );

    app.add_plugins((animation::plugin, movement::plugin));

    app.register_type::<XP>().register_type::<Level>();
    app.register_type::<Player>();

    app.add_systems(FixedUpdate, player_hit);

    app.add_observer(setup_player);
    app.add_observer(patch_player_spawn_pos);
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
    #[asset(path = "player_wizard_.png")]
    pub sprite: Handle<Image>,
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
    player_add: On<Add, Player>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    character: Query<&Characters, With<Player>>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    player_assets: If<Res<PlayerAssets>>,
    mut mesh: ResMut<Assets<Mesh>>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2 { x: 64, y: 64 }, 11, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);

    let mut player_sprite = player_assets.sprite.clone();

    if let Ok(character) = character.single() {
        player_sprite = match character {
            Characters::Wizzard => asset_server.load("player_wizard_.png"),
            Characters::Knight => asset_server.load("player_knight_.png"),
        };
    }

    commands.entity(player_add.entity).insert((
        Name::new("Player"),
        player_input_actions(),
        PlayerAnimation::new(),
        LockedAxes::ROTATION_LOCKED,
        Collider::circle(16.),
        LinearDamping(10.0),
        Friction::ZERO,
        //spawn_transform,
        Sprite::from_atlas_image(
            player_sprite,
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

    commands.trigger(PickUpWeaponEvent {
        kind: WeaponKind::Fireball,
    });

    commands.spawn((abilities::QAbility, abilities::heal::Heal));
    commands.spawn((abilities::EAbility, abilities::dash::Dash));
    commands.spawn((abilities::RAbility, abilities::summon::Summon));

    commands.trigger(PlayerSetupComplete);
}

fn patch_player_spawn_pos(
    add: On<Add, PlayerSpawnPoint>,
    mut player_pos: Query<&mut Transform, (With<Player>, Without<PlayerSpawnPoint>)>,
    player_spawn_query: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
) {
    let spawn_transform = *player_spawn_query.get(add.event().entity).unwrap();
    for mut player_pos in player_pos.iter_mut() {
        *player_pos = spawn_transform;
    }
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
            Action::<abilities::UseQAbility>::new(),
            bindings![KeyCode::KeyQ]
        ),
        (
            Action::<abilities::UseEAbility>::new(),
            bindings![KeyCode::KeyE]
        ),
        (
            Action::<abilities::UseRAbility>::new(),
            bindings![KeyCode::KeyR]
        )
    ])
}
