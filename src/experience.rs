use bevy::{ecs::entity, prelude::*, state::commands};

use crate::{
    PLAYER_SIZE, XP_GAIN_GEM,
    enemy::{EnemyDeathEvent, Speed},
    player::{Level, Player, XP, XpCollectionRange},
};

pub struct ExperiencePlugin;
impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collect_xp_gem)
            .add_observer(spawn_xp_gem)
            .add_observer(gain_xp)
            .add_observer(level_up);
    }
}

#[derive(Component)]
pub struct XpGem;

#[derive(Event)]
pub struct GainXpEvent;

#[derive(Event)]
pub struct LevelUpEvent(Entity);

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
            commands.trigger(GainXpEvent);
            commands.entity(gem_entity).despawn();
        }
    }

    Ok(())
}

fn gain_xp(
    trigger: Trigger<GainXpEvent>,
    player_q: Query<(&Level, &XP, Entity), With<Player>>,
    mut commands: Commands,
) -> Result {
    let (player_level, player_xp, player_entity) = player_q.single()?;

    player_xp.0 += XP_GAIN_GEM; //maybe increase with time

    if player_xp.0 == xp_for_level_up(player_level.0) {
        //Level Up
        commands.trigger(LevelUpEvent(player_entity));
        player_xp.0 = 0;
    }

    Ok(())
}

fn xp_for_level_up(current_level: i32) -> i32 {
    let base_xp = 100;

    let xp_needed = base_xp * current_level.isqrt();

    xp_needed
}

fn level_up(trigger: Trigger<LevelUpEvent>, mut player_q: Query<&mut Level, With<Player>>) {
    let player_entity = trigger.0;

    if let Ok(mut player_level) = player_q.get_mut(player_entity) {
        player_level.0 += 1;
    }
}
