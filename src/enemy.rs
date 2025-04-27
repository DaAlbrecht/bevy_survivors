use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSet, PLAYER_DMG_STAT,
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
                .in_set(AppSet::Update),
        );
        app.add_systems(
            Update,
            (
                enemy_movement,
                enemy_collision_detection,
                enemy_push_detection,
                update_enemy_timer,
                enemy_hit_detection,
            ),
        )
        .add_observer(enemy_pushing)
        .add_observer(enemy_collision_dmg)
        .add_observer(enemy_take_dmg);
    }
}

const SPAWN_RADIUS: f32 = 200.0;
const SEPARATION_RADIUS: f32 = 40.;
const SEPARATION_FORCE: f32 = 10.;
const ENEMY_DMG_STAT: f32 = 5.;

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct DamageCooldown {
    pub timer: Timer,
}

#[derive(Event)]
pub struct PlayerEnemyCollisionEvent(pub Entity);

#[derive(Event)]
pub struct PlayerPushingEvent(pub Entity);

#[derive(Event)]
pub struct EnemyHitEvent(pub Entity);

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
        Sprite {
            image: asset_server.load("Enemy.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
        Enemy,
        Health(10.),
        Speed(50.),
        DamageCooldown {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
    ));

    Ok(())
}

fn enemy_movement(
    enemy_query: Query<(&mut Transform, &Speed), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) -> Result {
    let player_transform = player_query.single()?;

    let enemy_positions = enemy_query
        .iter()
        .map(|t| t.0.translation)
        .collect::<Vec<Vec3>>();

    for (mut transform, speed) in enemy_query {
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

fn enemy_collision_detection(
    enemy_query: Query<(&mut Transform, Entity), With<Enemy>>,
    player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy) in &enemy_query {
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

        if distance_to_player <= SEPARATION_RADIUS {
            commands.trigger(PlayerEnemyCollisionEvent(enemy));
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

    // let player_pos = movement_controller.1.translation;

    for (mut enemy_pos, enemy_entity) in &mut enemy_query {
        if enemy_entity == push_entity {
            enemy_pos.translation += velocity.extend(0.0) * time.delta_secs();
        }
    }

    Ok(())
}

fn update_enemy_timer(time: Res<Time>, mut cooldowns: Query<&mut DamageCooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.timer.tick(time.delta());
    }
}

fn enemy_collision_dmg(
    trigger: Trigger<PlayerEnemyCollisionEvent>,
    mut player_health_q: Query<&mut Health, With<Player>>,
    mut enemy_dmg_timer_q: Query<&mut DamageCooldown, With<Enemy>>,
) -> Result {
    let mut player_health = player_health_q.single_mut()?;
    let enemy_entity = trigger.0;

    if let Ok(mut cooldown) = enemy_dmg_timer_q.get_mut(enemy_entity) {
        if cooldown.timer.finished() {
            player_health.0 -= ENEMY_DMG_STAT;
            info!("{:?}", player_health.0);
            cooldown.timer.reset();
        }
    }

    Ok(())
}

fn enemy_hit_detection(
    enemy_query: Query<(&Transform, Entity), (With<Enemy>, Without<PlayerSpell>)>,
    player_spell_query: Query<&Transform, (With<PlayerSpell>, Without<Player>)>,
    mut commands: Commands,
) {
    for &player_spell_pos in &player_spell_query {
        for (&enemy_pos, enemy_ent) in &enemy_query {
            if enemy_pos == player_spell_pos {
                commands.trigger(EnemyHitEvent(enemy_ent));
            }
        }
    }
}

fn enemy_take_dmg(
    trigger: Trigger<EnemyHitEvent>,
    mut enemy_q: Query<(&mut Health, Entity), With<Enemy>>,
) {
    let enemy_ent = trigger.0;
    for (mut enemy_health, entity) in &mut enemy_q {
        if enemy_ent == entity {
            enemy_health.0 -= PLAYER_DMG_STAT;
        }
    }
}
