use bevy::{
    color::palettes::css::{BLUE, GREY},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    ui::Val::{Percent, Px},
};

use super::Speed;
use super::enemy::EnemyDeathEvent;
use super::player::{Level, Player, XP, XpCollectionRange};
use crate::{PLAYER_SIZE, XP_GAIN_GEM, gameplay::overlays::Overlay, screens::Screen};

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(UiMaterialPlugin::<XpBarMaterial>::default());

    app.add_systems(OnEnter(Screen::Gameplay), spawn_xp_bar);
    app.add_systems(
        Update,
        (collect_xp_gem, update_xp_bar).run_if(in_state(Screen::Gameplay)),
    );

    app.world_mut().spawn((
        Observer::new(spawn_xp_gem),
        Name::new("spawn_xp_gem Observer"),
    ));
    app.world_mut()
        .spawn((Observer::new(gain_xp), Name::new("gain_xp Observer")));
    app.world_mut()
        .spawn((Observer::new(level_up), Name::new("level_up Observer")));
}

#[derive(Component, Reflect)]
pub(crate) struct XpGem;

#[derive(Event, Reflect)]
pub(crate) struct GainXpEvent;

#[derive(Event, Reflect)]
pub(crate) struct LevelUpEvent;

const BASE_LEVEL_XP: f32 = 100.;

fn spawn_xp_gem(
    trigger: On<EnemyDeathEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let enemy_pos = trigger.0.translation;

    commands.spawn((
        Name::new("XpGem"),
        Sprite {
            image: asset_server.load("xp_gem.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 0.),
        XpGem,
        Speed(200.),
    ));
}

fn collect_xp_gem(
    player_q: Query<(&Transform, &XpCollectionRange), With<Player>>,
    mut gem_q: Query<(&mut Transform, &Speed, Entity), (With<XpGem>, Without<Player>)>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
    let (player_position, collection_range) = player_q.single()?;

    for (mut gem_position, gem_speed, gem_entity) in &mut gem_q {
        if (player_position
            .translation
            .distance(gem_position.translation))
            <= collection_range.0
        {
            let direction = (player_position.translation - gem_position.translation).normalize();
            let movement = direction * (gem_speed.0 * time.delta_secs());
            gem_position.translation += movement;
        }

        if (player_position
            .translation
            .distance(gem_position.translation))
            <= PLAYER_SIZE / 2.0
        {
            commands.trigger(GainXpEvent);
            commands.entity(gem_entity).despawn();
        }
    }

    Ok(())
}

fn gain_xp(
    _trigger: On<GainXpEvent>,
    mut player_q: Query<(&Level, &mut XP), With<Player>>,
    mut commands: Commands,
) -> Result {
    let (player_level, mut player_xp) = player_q.single_mut()?;
    let xp_needed = BASE_LEVEL_XP * player_level.0.powf(2.);

    player_xp.0 += XP_GAIN_GEM;

    if player_xp.0 >= xp_needed {
        //Level Up
        commands.trigger(LevelUpEvent);
        player_xp.0 = 0.;
    }

    Ok(())
}

fn level_up(
    _trigger: On<LevelUpEvent>,
    mut player_q: Query<&mut Level, With<Player>>,
    mut next_state: ResMut<NextState<Overlay>>,
) -> Result {
    let mut level = player_q.single_mut()?;
    level.0 += 1.;

    next_state.set(Overlay::LevelUp);
    Ok(())
}

fn spawn_xp_bar(mut commands: Commands, mut ui_materials: ResMut<Assets<XpBarMaterial>>) {
    commands
        .spawn((
            Name::new("XP Bar"),
            Node {
                position_type: PositionType::Absolute,
                width: Percent(100.0),
                height: Px(40.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Percent(100.0),
                    height: Percent(50.0),
                    margin: UiRect::all(Px(20.)),
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                BorderRadius::all(Px(10.0)),
                MaterialNode(ui_materials.add(XpBarMaterial {
                    filled_color: BLUE.into(),
                    background_color: GREY.into(),
                    factor: 0.,
                    border_color: GREY.into(),
                    border_radius: Vec4::splat(20.),
                    offset: Vec4::splat(0.),
                })),
            ));
        });
}

fn update_xp_bar(
    player_q: Query<(&XP, &Level), With<Player>>,
    mut materials: ResMut<Assets<XpBarMaterial>>,
    xp_bar_material_q: Query<&MaterialNode<XpBarMaterial>>,
) -> Result {
    let (xp, level) = player_q.single()?;
    let xp_needed = BASE_LEVEL_XP * level.0.powf(2.);

    let factor = if xp_needed > 0. { xp.0 / xp_needed } else { 0. };

    let handle = xp_bar_material_q.single()?;

    if let Some(material) = materials.get_mut(handle) {
        material.factor = factor;
    }

    Ok(())
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct XpBarMaterial {
    #[uniform(0)]
    pub filled_color: LinearRgba,
    #[uniform(0)]
    pub background_color: LinearRgba,
    #[uniform(0)]
    pub factor: f32,
    #[uniform(0)]
    pub border_color: LinearRgba,
    #[uniform(0)]
    pub border_radius: Vec4,
    #[uniform(0)]
    pub offset: Vec4,
}

impl UiMaterial for XpBarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/xp_bar.wgsl".into()
    }
}
