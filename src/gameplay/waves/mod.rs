use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    PausableSystems, SPAWN_RADIUS, SPAWN_RADIUS_BUFFER,
    gameplay::{
        enemy::{
            Enemy, EnemyType,
            jumper::{JumperPatchEvent, JumperSpawnEvent},
            shooter::{ShooterPatchEvent, ShooterSpawnEvent},
            sprinter::{SprinterPatchEvent, SprinterSpawnEvent},
            walker::{WalkerPatchEvent, WalkerSpawnEvent},
        },
        player::Player,
        waves::waveplan::make_wave_plan,
    },
    screens::Screen,
};

mod waveplan;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), wave_spawner);

    app.add_systems(
        FixedUpdate,
        (wave_timer_handle)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_observer(patch_wave)
        .add_observer(account_enemies)
        .add_observer(enemy_patch_dispenser);
}

#[derive(Component)]
pub(crate) struct EnemyPool(pub HashMap<EnemyType, f32>);

#[derive(Component)]
pub(crate) struct SpritePool(pub HashMap<EnemyType, String>);

#[derive(Component)]
pub(crate) struct EnemyScreenCount(pub f32);

#[derive(Component)]
pub(crate) struct Wave;

#[derive(Component)]
pub(crate) struct SpawnTimer(pub Timer);

#[derive(Component)]
pub(crate) struct WaveDuration(pub Timer);

#[derive(Component)]
pub(crate) struct PowerLevel(pub f32);

#[derive(Event)]
pub(crate) struct EnemySpawnEvent;

#[derive(Event)]
pub(crate) struct WavePatchEvent;

#[derive(Event)]
pub(crate) struct EnemyPatchEvent;

#[derive(Resource)]
pub(crate) struct WaveStats {
    pub enemy_pool: HashMap<EnemyType, f32>,
    pub enemy_screen_count: f32,
    pub spawn_frequency: f32,
    pub duration: f32,
    pub power_level: f32,
    pub sprite_pool: HashMap<EnemyType, String>,
}

#[derive(Resource)]
pub(crate) struct WavePlan {
    pub waves: VecDeque<WaveStats>,
}

fn wave_spawner(mut commands: Commands) {
    info!("Wave spawned");
    commands.insert_resource(make_wave_plan());
    commands.spawn((Name::new("Wave"), Wave, DespawnOnExit(Screen::Gameplay)));
    // commands.spawn((Name::new("Wave"), Wave));
    commands.trigger(WavePatchEvent);
}

fn patch_wave(
    _trigger: On<WavePatchEvent>,
    mut commands: Commands,
    wave_q: Query<Entity, With<Wave>>,
    mut wave_plan: ResMut<WavePlan>,
) -> Result {
    let wave = wave_q.single()?;

    let stats = match wave_plan.waves.pop_front() {
        Some(stats) => stats,
        None => {
            info!("No wave left");
            return Ok(());
        }
    };

    commands.entity(wave).insert((
        EnemyPool(stats.enemy_pool),
        EnemyScreenCount(stats.enemy_screen_count),
        SpawnTimer(Timer::from_seconds(
            1.0 / stats.spawn_frequency,
            TimerMode::Once,
        )),
        WaveDuration(Timer::from_seconds(stats.duration, TimerMode::Once)),
        PowerLevel(stats.power_level),
        SpritePool(stats.sprite_pool),
    ));

    info!("Wave patched");
    Ok(())
}

fn account_enemies(
    _trigger: On<EnemySpawnEvent>,
    mut wave_q: Query<(&EnemyPool, &EnemyScreenCount), With<Wave>>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<(&Transform, &EnemyType), With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let (enemy_pool, screen_count) = wave_q.single_mut()?;
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
        if enemy_pos.distance(player_pos.translation.truncate())
            <= (SPAWN_RADIUS + SPAWN_RADIUS_BUFFER)
        {
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
    mut wave_q: Query<(&mut SpawnTimer, &mut WaveDuration), With<Wave>>,
    time: Res<Time>,
) {
    for (mut spawn_timer, mut wave_timer) in &mut wave_q {
        spawn_timer.0.tick(time.delta());
        wave_timer.0.tick(time.delta());

        if spawn_timer.0.is_finished() {
            commands.trigger(EnemySpawnEvent);
            spawn_timer.0.reset();
        }

        if wave_timer.0.is_finished() {
            commands.trigger(WavePatchEvent);
            commands.trigger(EnemyPatchEvent);
            wave_timer.0.reset();
        }
    }
}

fn enemy_patch_dispenser(
    _trigger: On<EnemyPatchEvent>,
    mut commands: Commands,
    wave_q: Query<(&EnemyPool, &PowerLevel, &SpritePool), With<Wave>>,
) -> Result {
    let (enemy_pool, power_level, sprite_pool) = wave_q.single()?;

    for enemy_type in enemy_pool.0.keys() {
        let Some(sprite) = sprite_pool.0.get(enemy_type) else {
            continue;
        };
        match enemy_type {
            EnemyType::Walker => {
                commands.trigger(WalkerPatchEvent(power_level.0, sprite.to_string()))
            }
            EnemyType::Shooter => {
                commands.trigger(ShooterPatchEvent(power_level.0, sprite.to_string()))
            }
            EnemyType::Sprinter => {
                commands.trigger(SprinterPatchEvent(power_level.0, sprite.to_string()))
            }
            EnemyType::Jumper => {
                commands.trigger(JumperPatchEvent(power_level.0, sprite.to_string()))
            }
            EnemyType::None => (),
        }
    }

    Ok(())
}
