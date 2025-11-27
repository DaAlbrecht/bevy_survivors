use crate::{
    audio::SfxPool,
    gameplay::{
        Health,
        enemy::{Colliding, Enemy},
        healthbar::HealthBarMaterial,
        player::{Player, PlayerHitEvent},
    },
};
use avian2d::prelude::{CollisionEnd, CollisionStart};
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

pub(crate) fn player_hit(
    trigger: On<PlayerHitEvent>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    mut player_q: Query<&mut Health, With<Player>>,
    healthbar_material_q: Query<&MeshMaterial2d<HealthBarMaterial>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut health = player_q.single_mut()?;
    health.0 -= trigger.dmg;
    info!("attacking player, player_health: {}", health.0);

    commands.spawn((
        SamplePlayer::new(asset_server.load("audio/sound_effects/impact_1.ogg")),
        SfxPool,
    ));
    let per = health.0 / 100.;

    let handle = healthbar_material_q.single()?.clone();
    let material = health_bar_materials.get_mut(&handle).unwrap();
    material.percent = per;

    Ok(())
}

pub(crate) fn player_collision_start(
    event: On<CollisionStart>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    let Some(enemy) = [event.collider1, event.collider2]
        .into_iter()
        .find(|&e| enemy_q.contains(e))
    else {
        return;
    };

    commands.entity(enemy).insert((Colliding,));
}

pub(crate) fn player_collision_end(
    event: On<CollisionEnd>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    let Some(enemy) = [event.collider1, event.collider2]
        .into_iter()
        .find(|&e| enemy_q.contains(e))
    else {
        return;
    };

    commands.entity(enemy).remove::<Colliding>();
}
