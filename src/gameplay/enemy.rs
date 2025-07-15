use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystem, ENEMY_SIZE, PLAYER_DMG_STAT, SPELL_SIZE,
    gameplay::player::{Direction, Knockback},
    screens::Screen,
};

use super::{
    movement::MovementController,
    player::{Player, PlayerSpell},
};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_enemy
                .run_if(on_timer(Duration::from_millis(2000)))
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSystem::Update),
        );
        app.add_systems(
            Update,
            (
                enemy_movement,
                enemy_colliding_detection,
                enemy_stop_colliding_detection,
                enemy_push_detection,
                enemy_hit_detection,
                move_enemy_from_knockback,
                attack,
            )
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_observer(enemy_pushing)
        .add_observer(enemy_take_dmg)
        .add_observer(enemy_get_pushed_from_hit);
    }
}

const SPAWN_RADIUS: f32 = 200.0;
const SEPARATION_RADIUS: f32 = 40.;
const SEPARATION_FORCE: f32 = 10.;
const ENEMY_DMG_STAT: f32 = 5.;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
#[require(Health(10.), Speed(50.), DamageCooldown, Sprite, Transform)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component, Default)]
pub struct DamageCooldown(pub Timer);

#[derive(Event)]
pub struct PlayerPushingEvent(pub Entity);

#[derive(Event)]
pub struct EnemyHitEvent {
    pub entity_hit: Entity,
    pub spell_entity: Entity,
}
#[derive(Event)]
pub struct EnemyDeathEvent(pub Transform);

#[derive(Component)]
pub struct Colliding;

//type shenanigans
#[derive(Component)]
pub struct KnockbackDirection(pub Direction);

fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.gen_range(0.0..(2. * PI));
    let random_radius: f32 = rng.gen_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    commands.spawn((
        Enemy,
        Sprite {
            image: asset_server.load("Enemy.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
        KnockbackDirection(Direction(Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        })),
        Knockback(0.0),
    ));

    Ok(())
}

fn enemy_movement(
    enemy_query: Query<(&mut Transform, &Speed, &Knockback), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) -> Result {
    let player_transform = player_query.single()?;

    let enemy_positions = enemy_query
        .iter()
        .map(|t| t.0.translation)
        .collect::<Vec<Vec3>>();

    for (mut transform, speed, knockback) in enemy_query {
        if knockback.0 > 1.0 {
            //skip movement if enemy gets knockedback
            continue;
        }
        let direction = (player_transform.translation - transform.translation).normalize();

        // Separation force calculation for enemies
        let mut separation_force = Vec3::ZERO;
        for &other_pos in &enemy_positions {
            // skip ourselves
            if other_pos == transform.translation {
                continue;
            }
            // Check if the distance between enemy `A` and all other enemies is less than the
            // `SEPARATION_RADIUS`. If so, push enemy `A` away from the other enemy to maintain spacing.
            let distance = transform.translation.distance(other_pos);
            if distance < SEPARATION_RADIUS {
                let push_dir = (transform.translation - other_pos).normalize();
                let push_strength = (SEPARATION_RADIUS - distance) / SEPARATION_RADIUS;
                separation_force += push_dir * push_strength * SEPARATION_FORCE;
            }
        }
        // Separation force calculation for the player
        let distance_to_player = transform.translation.distance(player_transform.translation);
        if distance_to_player < SEPARATION_RADIUS {
            let push_dir = (transform.translation - player_transform.translation).normalize();
            let push_strength = (SEPARATION_RADIUS - distance_to_player) / SEPARATION_RADIUS;
            separation_force += push_dir * push_strength * SEPARATION_FORCE;
        }

        let movement = (direction + separation_force).normalize() * (speed.0 * time.delta_secs());
        transform.translation += movement;
    }
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
    movement_query: Query<&MovementController, With<Player>>,
    mut enemy_query: Query<(&mut Transform, Entity), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
) -> Result {
    let push_entity = trigger.event().0;

    let movement_controller = movement_query.single()?;
    let velocity = movement_controller.max_speed * movement_controller.intent;

    for (mut enemy_pos, enemy_entity) in &mut enemy_query {
        if enemy_entity == push_entity {
            enemy_pos.translation += velocity.extend(0.0) * time.delta_secs();
        }
    }

    Ok(())
}

fn attack(
    time: Res<Time>,
    mut player_health_q: Query<&mut Health, With<Player>>,
    mut enemy_dmg_timer_q: Query<&mut DamageCooldown, (With<Enemy>, With<Colliding>)>,
) -> Result {
    for mut timer in enemy_dmg_timer_q.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            let mut player_health = player_health_q.single_mut()?;
            player_health.0 -= ENEMY_DMG_STAT;
            debug!("attacking player, player_health: {}", player_health.0);
        }
    }

    Ok(())
}

fn enemy_hit_detection(
    enemy_query: Query<(&Transform, Entity), (With<Enemy>, Without<PlayerSpell>)>,
    player_spell_query: Query<(&Transform, Entity), (With<PlayerSpell>, Without<Player>)>,
    mut commands: Commands,
) {
    for (&player_spell_pos, spell_entity) in &player_spell_query {
        for (&enemy_pos, enemy_entity) in &enemy_query {
            if (player_spell_pos.translation.distance(enemy_pos.translation) - (SPELL_SIZE / 2.0))
                <= ENEMY_SIZE / 2.0
            {
                commands.trigger(EnemyHitEvent {
                    entity_hit: enemy_entity,
                    spell_entity,
                });
            }
        }
    }
}

fn enemy_take_dmg(
    trigger: Trigger<EnemyHitEvent>,
    mut enemy_q: Query<(&mut Health, &Transform), With<Enemy>>,
    mut commands: Commands,
) {
    let enemy_entity = trigger.entity_hit;
    let spell_entity = trigger.spell_entity;

    if let Ok((mut health, transform)) = enemy_q.get_mut(enemy_entity) {
        health.0 -= PLAYER_DMG_STAT;
        if health.0 <= 0.0 {
            commands.entity(enemy_entity).despawn();
            commands.trigger(EnemyDeathEvent(*transform));
        }
        commands.entity(spell_entity).despawn();
    }
}

fn enemy_get_pushed_from_hit(
    trigger: Trigger<EnemyHitEvent>,
    mut enemy_q: Query<(&mut Knockback, &mut KnockbackDirection), With<Enemy>>,
    spell_q: Query<(&Direction, &Knockback), (With<PlayerSpell>, Without<Enemy>)>,
) {
    let enemy_entity = trigger.entity_hit;
    let spell_entity = trigger.spell_entity;

    if let Ok((spell_direction, spell_knockback)) = spell_q.get(spell_entity) {
        if let Ok((mut enemy_knockback, mut enemy_knockback_direction)) =
            enemy_q.get_mut(enemy_entity)
        {
            enemy_knockback.0 = spell_knockback.0;
            //type shenanigans continue
            enemy_knockback_direction.0.0 = spell_direction.0;
        }
    }
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
