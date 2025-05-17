use bevy::prelude::*;

use crate::{
    PLAYER_SIZE,
    enemy::{EnemyDeathEvent, Speed},
    player::{Player, XpCollectionRange},
};

pub struct ExperiencePlugin;
impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collect_xp_gem)
            .add_observer(spawn_xp_gem)
            .add_observer(gain_xp);
    }
}

#[derive(Component)]
pub struct XpGem;

#[derive(Event)]
pub struct GainXpEvent;

fn spawn_xp_gem(
    trigger: Trigger<EnemyDeathEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let enemy_pos = trigger.0.translation;

    commands.spawn((
        Name::new("XpGem"),
        Sprite {
            image: asset_server.load("XP_GEM.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 0.),
        XpGem,
        Speed(200.),
    ));

    Ok(())
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
            info!("Gem Hit");
            commands.trigger(GainXpEvent);
            commands.entity(gem_entity).despawn_related::<Children>();
        }
    }

    Ok(())
}

fn gain_xp(trigger: Trigger<GainXpEvent>) {}
