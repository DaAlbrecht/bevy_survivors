use crate::{
    audio::SfxPool,
    gameplay::{
        Health,
        enemy::{DamageCooldown, Enemy},
        healthbar::HealthBarMaterial,
        player::Player,
        weapons::Damage,
    },
};
use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

pub(crate) fn player_hit(
    time: Res<Time>,
    mut commands: Commands,
    mut player_q: Query<(&mut Health, &CollidingEntities), With<Player>>,
    mut enemy_dmg_timer_q: Query<(&mut DamageCooldown, &Damage), With<Enemy>>,
    healthbar_material_q: Query<&MeshMaterial2d<HealthBarMaterial>>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    asset_server: Res<AssetServer>,
) -> Result {
    for (mut player_health, colliding_entities) in &mut player_q {
        for colliding_entity in colliding_entities.iter() {
            // If the colliding entity is not an Enemy, skip this collider
            let Ok((mut timer, damage)) = enemy_dmg_timer_q.get_mut(*colliding_entity) else {
                continue;
            };

            if timer.0.tick(time.delta()).just_finished() {
                player_health.0 -= damage.0;
                info!("attacking player, player_health: {}", player_health.0);

                commands.spawn((
                    SamplePlayer::new(asset_server.load("audio/sound_effects/impact_1.ogg")),
                    SfxPool,
                ));
                let per = player_health.0 / 100.;

                let handle = healthbar_material_q.single()?.clone();
                let material = health_bar_materials.get_mut(&handle).unwrap();
                material.percent = per;
            }
        }
    }

    Ok(())
}
