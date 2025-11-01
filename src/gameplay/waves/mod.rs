use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    SPAWN_RADIUS, SPAWN_RADIUS_BUFFER,
    gameplay::{
        enemy::{
            Enemy, EnemyType, jumper::JumperSpawnEvent, shooter::ShooterSpawnEvent,
            sprinter::SprinterSpawnEvent, walker::WalkerSpawnEvent,
        },
        player::Player,
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (wave_spawner, patch_wave.after(wave_spawner)),
    );

    app.add_systems(
        FixedUpdate,
        (wave_timer_handle).run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(account_enemies);
}

#[derive(Component)]
pub(crate) struct EnemyPool(pub HashMap<EnemyType, f32>);

#[derive(Component)]
pub(crate) struct EnemyScreenCount(pub f32);

#[derive(Component, PartialEq, Debug)]
pub(crate) enum WaveType {
    Early,
    Mid,
    Late,
}

#[derive(Component)]
pub(crate) struct Active;

#[derive(Component)]
pub(crate) struct Wave;

#[derive(Component)]
pub(crate) struct SpawnTimer(pub Timer);

#[derive(Component)]
pub(crate) struct WaveDuration(pub Timer);

#[derive(Event)]
pub(crate) struct EnemySpawnEvent;

fn wave_spawner(mut commands: Commands) {
    commands.spawn((Name::new("Earlywave"), Wave, WaveType::Early));
    commands.spawn((Name::new("Midwave"), Wave, WaveType::Mid));
    commands.spawn((Name::new("Latewave"), Wave, WaveType::Late));
}

fn patch_wave(mut wave_q: Query<(Entity, &WaveType), With<Wave>>, mut commands: Commands) {
    for (wave, wave_type) in &mut wave_q {
        match wave_type {
            WaveType::Early => {
                commands.entity(wave).insert(EnemyPool(HashMap::from([
                    (EnemyType::Walker, 0.8),
                    (EnemyType::Jumper, 0.2),
                ])));
                commands.entity(wave).insert(EnemyScreenCount(10.0));
                commands
                    .entity(wave)
                    .insert(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));
                commands
                    .entity(wave)
                    .insert(WaveDuration(Timer::from_seconds(
                        60.0 * 0.5,
                        TimerMode::Once,
                    )));
                commands.entity(wave).insert(Active);
            }
            WaveType::Mid => {
                commands
                    .entity(wave)
                    .insert(EnemyPool(HashMap::from([(EnemyType::Sprinter, 1.0)])));
                commands.entity(wave).insert(EnemyScreenCount(20.0));
                commands
                    .entity(wave)
                    .insert(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));
                commands
                    .entity(wave)
                    .insert(WaveDuration(Timer::from_seconds(
                        60.0 * 0.5,
                        TimerMode::Once,
                    )));
            }
            WaveType::Late => {
                commands
                    .entity(wave)
                    .insert(EnemyPool(HashMap::from([(EnemyType::Shooter, 1.0)])));
                commands.entity(wave).insert(EnemyScreenCount(30.0));
                commands
                    .entity(wave)
                    .insert(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));
                commands
                    .entity(wave)
                    .insert(WaveDuration(Timer::from_seconds(
                        60.0 * 0.5,
                        TimerMode::Once,
                    )));
            }
        }
    }
}

fn account_enemies(
    _trigger: On<EnemySpawnEvent>,
    mut wave_q: Query<(&EnemyPool, &EnemyScreenCount), With<Active>>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<(&Transform, &EnemyType), With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let (enemy_pool, screen_count) = wave_q.single_mut()?;
    let player_pos = player_q.single()?.translation.truncate();
    let mut live_enemies: HashMap<EnemyType, f32> = HashMap::new();
    let mut absolut_enemy_count = 0.0;

    //Populate live enemies map
    for (enemy_type, _) in &enemy_pool.0 {
        live_enemies.entry(*enemy_type).or_insert(0.0);
    }

    //Spawn first enemies
    if enemy_q.is_empty() {
        let (mut signature_enemy, mut count) = (EnemyType::None, 0.0);
        for (enemy_type, enemy_count) in &enemy_pool.0 {
            if count < *enemy_count {
                (signature_enemy, count) = (*enemy_type, *enemy_count);
            }
        }

        match signature_enemy {
            EnemyType::Walker => commands.trigger(WalkerSpawnEvent),
            EnemyType::Shooter => commands.trigger(ShooterSpawnEvent),
            EnemyType::Sprinter => commands.trigger(SprinterSpawnEvent),
            EnemyType::Jumper => commands.trigger(JumperSpawnEvent),
            EnemyType::None => (),
        }
        return Ok(());
    }

    // Count enemies
    for (transform, enemy_type) in &enemy_q {
        let enemy_pos = transform.translation.truncate();
        if enemy_pos.distance(player_pos) <= (SPAWN_RADIUS + SPAWN_RADIUS_BUFFER) {
            absolut_enemy_count += 1.0;
            if let Some(count) = live_enemies.get_mut(enemy_type) {
                *count += 1.0
            }
        }
    }

    //Make values relative to absolut enemy count
    for (_, count) in &mut live_enemies {
        *count /= absolut_enemy_count;
    }

    //Spawn if there are not enough enemies alive
    if absolut_enemy_count < screen_count.0 {
        let (mut demanded_type, mut count_diff) = (&EnemyType::None, 0.0);

        //Get enemy type with biggest diff the pool
        for (enemy_type, count) in &live_enemies {
            if let Some(pool_count) = enemy_pool.0.get(enemy_type) {
                let diff = pool_count - count;
                if diff >= count_diff {
                    (demanded_type, count_diff) = (enemy_type, diff)
                }
            }
        }

        match demanded_type {
            EnemyType::Walker => commands.trigger(WalkerSpawnEvent),
            EnemyType::Shooter => commands.trigger(ShooterSpawnEvent),
            EnemyType::Sprinter => commands.trigger(SprinterSpawnEvent),
            EnemyType::Jumper => commands.trigger(JumperSpawnEvent),
            EnemyType::None => (),
        }
    }

    Ok(())
}

fn wave_timer_handle(
    mut commands: Commands,
    mut active_wave_q: Query<(Entity, &WaveType, &mut SpawnTimer, &mut WaveDuration), With<Active>>,
    wave_q: Query<(Entity, &WaveType), With<Wave>>,
    time: Res<Time>,
) {
    for (current_wave, current_wave_type, mut spawn_timer, mut wave_timer) in &mut active_wave_q {
        spawn_timer.0.tick(time.delta());
        wave_timer.0.tick(time.delta());

        if spawn_timer.0.is_finished() {
            commands.trigger(EnemySpawnEvent);
            spawn_timer.0.reset();
        }

        if wave_timer.0.is_finished() {
            //Dont like this yet
            let next_wave_type = match current_wave_type {
                WaveType::Early => WaveType::Mid,
                WaveType::Mid => WaveType::Late,
                WaveType::Late => WaveType::Early,
            };
            for (wave, wave_type) in &wave_q {
                if *wave_type == next_wave_type {
                    //Later we can despawn for now we need it so we can return from late to early
                    commands.entity(current_wave).remove::<Active>();
                    commands.entity(wave).insert(Active);
                    info!("Wavechange, new Wave: {:?}", wave_type);
                }
            }

            wave_timer.0.reset();
        }
    }
}
