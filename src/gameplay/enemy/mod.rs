use std::f32::consts::PI;

use bevy_enhanced_input::action::Action;

use bevy::prelude::*;
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    gameplay::{
        Health,
        player::{Direction, Move, PlayerHitEvent},
        spells::{CastSpell, Despawn, Spell},
    },
    screens::Screen,
};

use super::player::Player;

use super::spells::{Knockback, PlayerProjectile};

mod walker;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(walker::plugin);

    app.add_systems(
        Update,
        (
            enemy_colliding_detection,
            enemy_stop_colliding_detection,
            enemy_push_detection,
            move_enemy_from_knockback,
            attack,
            enemy_despawner,
        )
            .run_if(in_state(Screen::Gameplay)),
    )
    .add_observer(enemy_pushing)
    .add_observer(enemy_take_dmg)
    .add_observer(enemy_get_pushed_from_hit);
}

const SPAWN_RADIUS: f32 = 200.0;
const SEPARATION_RADIUS: f32 = 40.;
const SEPARATION_FORCE: f32 = 10.;
const ENEMY_DMG_STAT: f32 = 5.;

#[derive(Component)]
pub(crate) struct Speed(pub f32);

#[derive(Component, Default)]
pub(crate) struct DamageCooldown(pub Timer);

#[derive(Component, Default)]
pub(crate) struct Enemy;

#[derive(Event)]
pub(crate) struct PlayerPushingEvent(pub Entity);

#[derive(Event)]
pub(crate) struct EnemyDamageEvent {
    pub entity_hit: Entity,
    pub dmg: f32,
}

#[derive(Event)]
pub(crate) struct EnemyKnockbackEvent {
    pub entity_hit: Entity,
    pub spell_entity: Entity,
}

#[derive(Event)]
pub(crate) struct EnemyDeathEvent(pub Transform);

#[derive(Component)]
pub(crate) struct Colliding;

//type shenanigans
#[derive(Component)]
pub(crate) struct KnockbackDirection(pub Direction);

fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    commands.spawn((
        Name::new("Default Enemy"),
        Enemy,
        Sprite {
            image: asset_server.load("enemies/Walker.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));

    Ok(())
}

fn enemy_colliding_detection(
    enemy_query: Query<(&mut Transform, Entity), (With<Enemy>, Without<Colliding>)>,
    player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy) in &enemy_query {
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

        if distance_to_player <= SEPARATION_RADIUS {
            commands.entity(enemy).insert(Colliding);
        }
    }
    Ok(())
}

fn enemy_stop_colliding_detection(
    enemy_query: Query<(&mut Transform, Entity), (With<Enemy>, With<Colliding>)>,
    player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy) in &enemy_query {
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

        if distance_to_player > SEPARATION_RADIUS {
            commands.entity(enemy).remove::<Colliding>();
        }
    }
    Ok(())
}

fn enemy_push_detection(
    enemy_query: Query<(&mut Transform, Entity), With<Enemy>>,
    player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy) in &enemy_query {
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

        if distance_to_player <= SEPARATION_RADIUS - 5.0 {
            commands.trigger(PlayerPushingEvent(enemy));
        }
    }
    Ok(())
}

fn enemy_pushing(
    trigger: Trigger<PlayerPushingEvent>,
    move_action: Single<&Action<Move>>,
    mut enemy_query: Query<(&mut Transform, Entity), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
) {
    let push_entity = trigger.event().0;

    for (mut enemy_pos, enemy_entity) in &mut enemy_query {
        if enemy_entity == push_entity {
            enemy_pos.translation += move_action.extend(0.0) * time.delta_secs();
        }
    }
}

fn attack(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_dmg_timer_q: Query<&mut DamageCooldown, (With<Enemy>, With<Colliding>)>,
) {
    for mut timer in &mut enemy_dmg_timer_q {
        if timer.0.tick(time.delta()).just_finished() {
            commands.trigger(PlayerHitEvent {
                dmg: ENEMY_DMG_STAT,
            });
        }
    }
}

fn enemy_take_dmg(
    trigger: Trigger<EnemyDamageEvent>,
    mut enemy_q: Query<(&mut Health, &Transform), (With<Enemy>, Without<Despawn>)>,
    mut commands: Commands,
) {
    let enemy_entity = trigger.entity_hit;

    if let Ok((mut health, transform)) = enemy_q.get_mut(enemy_entity) {
        health.0 -= trigger.dmg;
        if health.0 <= 0.0 {
            commands.entity(enemy_entity).insert(Despawn);
            commands.trigger(EnemyDeathEvent(*transform));
        }
    }
}

fn enemy_get_pushed_from_hit(
    trigger: Trigger<EnemyKnockbackEvent>,
    mut enemy_q: Query<(&mut Knockback, &mut KnockbackDirection), With<Enemy>>,
    knockback: Query<&Knockback, (With<Spell>, Without<Enemy>)>,
    projectile_direction: Query<&Direction, With<PlayerProjectile>>,
    spells: Query<&CastSpell>,
) -> Result {
    let enemy_entity = trigger.entity_hit;
    let projectile_entity = trigger.spell_entity;

    //Get the Spell of this projectile, each projectile, this is a 1-many relationship
    let spell = spells
        .iter_ancestors(projectile_entity)
        .next()
        .expect("there should always only be one ancestor spell for each projectile");

    let knockback = knockback.get(spell)?.0;

    let direction = projectile_direction.get(projectile_entity)?.0;

    if let Ok((mut enemy_knockback, mut enemy_knockback_direction)) = enemy_q.get_mut(enemy_entity)
    {
        enemy_knockback.0 = knockback;
        //type shenanigans continue
        enemy_knockback_direction.0.0 = direction;
    }

    Ok(())
}

fn move_enemy_from_knockback(
    mut enemy_q: Query<(&mut Knockback, &mut Transform, &KnockbackDirection), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut enemy_knockback, mut enemy_transform, enemy_direction) in &mut enemy_q {
        if enemy_knockback.0 > 0.0 {
            //Very sorry for the type shenanigans at this point tbh
            enemy_transform.translation +=
                enemy_knockback.0 * enemy_direction.0.0 * time.delta_secs();

            //reduce knockback speed each frame (friction)
            enemy_knockback.0 *= 0.95;

            //Stop if slow
            if enemy_knockback.0 <= 1.0 {
                enemy_knockback.0 = 0.0;
            }
        }
    }
}

fn enemy_despawner(enemy_q: Query<Entity, (With<Enemy>, With<Despawn>)>, mut commands: Commands) {
    for enemy in &enemy_q {
        commands.entity(enemy).despawn();
    }
}
