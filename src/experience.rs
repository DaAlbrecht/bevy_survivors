use bevy::prelude::*;

use crate::{
    PLAYER_SIZE, XP_GAIN_GEM,
    enemy::{EnemyDeathEvent, Speed},
    player::{Level, Player, XP, XpCollectionRange},
    screens::Screen,
};

pub struct ExperiencePlugin;
impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collect_xp_gem);

        app.world_mut().spawn((
            Observer::new(spawn_xp_gem),
            Name::new("spawn_xp_gem Observer"),
        ));
        app.world_mut()
            .spawn((Observer::new(gain_xp), Name::new("gain_xp Observer")));
        app.world_mut()
            .spawn((Observer::new(level_up), Name::new("level_up Observer")));
    }
}

#[derive(Component)]
pub struct XpGem;

#[derive(Event)]
pub struct GainXpEvent;

#[derive(Event)]
pub struct LevelUpEvent;

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
    _trigger: Trigger<GainXpEvent>,
    mut player_q: Query<(&Level, &mut XP), With<Player>>,
    mut commands: Commands,
) -> Result {
    let base_xp = 100;
    let (player_level, mut player_xp) = player_q.single_mut()?;
    let xp_needed = base_xp * player_level.0.pow(2);

    player_xp.0 += XP_GAIN_GEM; //maybe increase with time   

    if player_xp.0 >= xp_needed {
        //Level Up
        commands.trigger(LevelUpEvent);
        player_xp.0 = 0;
    }

    Ok(())
}

fn level_up(
    _trigger: Trigger<LevelUpEvent>,
    mut player_q: Query<&mut Level, With<Player>>,
    mut next_state: ResMut<NextState<Screen>>,
) -> Result {
    let mut level = player_q.single_mut()?;
    level.0 += 1;

    next_state.set(Screen::LevelUp);
    Ok(())
}
