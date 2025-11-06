use bevy::{
    color::palettes::tailwind,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    sprite_render::MeshMaterial2d,
};
use bevy_seedling::sample::AudioSample;

use crate::{
    PausableSystems, PrePhysicsAppSystems,
    asset_tracking::LoadResource,
    fixed_update_inspection::did_fixed_update_happen,
    gameplay::{
        Health, healthbar::HealthBarMaterial, movement::MovementController, player::hit::player_hit,
    },
};

pub(crate) mod animation;
pub(crate) mod hit;
pub(crate) mod movement;
use animation::PlayerAnimation;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((animation::plugin, movement::plugin));

    app.load_resource::<PlayerAssets>();

    app.add_observer(player_hit);

    app.register_type::<XP>().register_type::<Level>();

    app.add_systems(
        RunFixedMainLoop,
        record_player_directional_input
            .in_set(PausableSystems)
            .in_set(PrePhysicsAppSystems::AccumulateInput),
    );
}

/// The player character.
pub fn player(
    speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    health_bar_materials: &mut Assets<HealthBarMaterial>,
    mesh: &mut Assets<Mesh>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite::from_atlas_image(
            player_assets.ducky.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            },
        ),
        Transform::from_xyz(50., 0., 0.),
        MovementController {
            speed,
            ..default()
        },
        player_animation,
        children![(
            Mesh2d(mesh.add(Rectangle::new(32.0, 5.0))),
            MeshMaterial2d(health_bar_materials.add(HealthBarMaterial {
                foreground_color: tailwind::GREEN_300.into(),
                background_color: tailwind::RED_300.into(),
                percent: 1.,
            })),
            Transform::from_xyz(0.0, -25.0, 0.),
        )],
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(
    Health(100.),
    XpCollectionRange(150.0),
    XP(0.),
    Level(1.),
    MovementController { speed: 400.0, ..default()}
)]
pub(crate) struct Player;

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

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSample>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings("player.png", |settings: &mut ImageLoaderSettings| {
                // Use `nearest` image sampling to preserve pixel art style.
                settings.sampler = ImageSampler::nearest();
            }),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
