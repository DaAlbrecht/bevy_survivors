use crate::{
    audio::SfxPool,
    gameplay::{
        Health,
        healthbar::HealthBarMaterial,
        player::{Player, PlayerHitEvent},
    },
};
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
        SamplePlayer::new(asset_server.load("audio/sound_effects/hit.wav")),
        SfxPool,
    ));
    let per = health.0 / 100.;

    let handle = healthbar_material_q.single()?.clone();
    let material = health_bar_materials.get_mut(&handle).unwrap();
    material.percent = per;

    Ok(())
}
