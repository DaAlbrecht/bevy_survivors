use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

use crate::{
    PausableSystems,
    audio::SfxPool,
    gameplay::{
        Speed,
        damage_numbers::DamageType,
        enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
        player::{Direction, Level, Player},
        simple_animation::{AnimationIndices, AnimationTimer},
        weapons::{
            CastWeapon, Cooldown, Damage, Halt, PlayerProjectile, ProjectileCount, Weapon,
            WeaponAttackEvent, WeaponPatchEvent, WeaponType, weaponstats::EnergyLevels,
        },
    },
    screens::Screen,
};

#[derive(Component)]
#[require(Weapon, WeaponType::Energy, Name::new("Energy Weapon"))]
#[derive(Reflect)]
pub(crate) struct Energy;

#[derive(Component, Reflect)]
pub(crate) struct EnergyProjectile;

#[derive(Event, Reflect)]
pub(crate) struct EnergyAttackEvent;

#[derive(Component)]
struct StaggeredSpawn {
    delay: Timer,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        release_staggered_projectiles
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

pub fn patch_energy(
    _trigger: On<WeaponPatchEvent>,
    mut commands: Commands,
    weapon_q: Query<Entity, With<Energy>>,
    mut weapon_levels: ResMut<EnergyLevels>,
) -> Result {
    let weapon = weapon_q.single()?;

    let Some(stats) = weapon_levels.levels.pop_front() else {
        return Ok(());
    };

    commands
        .entity(weapon)
        .insert(Level(stats.level))
        .insert(Damage(stats.damage))
        .insert(Speed(stats.speed))
        .insert(ProjectileCount(stats.projectile_count))
        .insert(Cooldown(Timer::from_seconds(
            stats.cooldown,
            TimerMode::Once,
        )));

    info!("{:} Level Up", weapon);

    Ok(())
}

pub fn spawn_energy_projectiles(
    _trigger: On<WeaponAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    energy_q: Query<(Entity, &ProjectileCount), With<Energy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) -> Result {
    let Ok(player_transform) = player_q.single() else {
        return Ok(());
    };

    let (energy, projectile_count) = energy_q.single()?;

    let texture = asset_server.load("fx/energy.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);

    let num_waves = projectile_count.0 as usize;
    let projectiles_per_wave = 12;
    let angle_step = std::f32::consts::TAU / projectiles_per_wave as f32;
    let wave_delay = 0.3;
    let stagger_delay = 0.02;

    for wave in 0..num_waves {
        for i in 0..projectiles_per_wave {
            let angle = angle_step * i as f32;
            let direction = Vec2::new(angle.cos(), angle.sin());

            let towards_quaternion =
                Quat::from_rotation_arc(Vec3::Y, direction.extend(0.).normalize());

            commands
                .spawn((
                    Name::new("energy projectile"),
                    Sprite::from_atlas_image(
                        texture.clone(),
                        TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index: 0,
                        },
                    ),
                    AnimationIndices { first: 0, last: 3 },
                    AnimationTimer {
                        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    },
                    CastWeapon(energy),
                    Transform::from_xyz(
                        player_transform.translation.x,
                        player_transform.translation.y,
                        10.0,
                    )
                    .with_rotation(towards_quaternion),
                    Direction(direction.extend(0.)),
                    EnergyProjectile,
                    PlayerProjectile,
                    Halt,
                    StaggeredSpawn {
                        delay: Timer::from_seconds(
                            wave as f32 * wave_delay + i as f32 * stagger_delay,
                            TimerMode::Once,
                        ),
                    },
                ))
                .observe(on_energy_hit);

            commands.spawn((
                SamplePlayer::new(asset_server.load("audio/sound_effects/energy_cast.wav")),
                SfxPool,
            ));
        }
    }

    Ok(())
}

fn release_staggered_projectiles(
    mut projectile_q: Query<(Entity, &mut StaggeredSpawn), With<EnergyProjectile>>,
    mut commands: Commands,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta();

    for (entity, mut stagger) in &mut projectile_q {
        stagger.delay.tick(dt);

        if stagger.delay.just_finished() {
            commands.entity(entity).remove::<Halt>();
            commands.entity(entity).remove::<StaggeredSpawn>();
        }
    }
}

fn on_energy_hit(
    event: On<avian2d::prelude::CollisionStart>,
    enemy_q: Query<Entity, With<Enemy>>,
    projectile_q: Query<&CastWeapon, With<EnergyProjectile>>,
    weapon_q: Query<&Damage, With<Energy>>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;
    let enemy = event.collider2;

    let Ok(cast_weapon) = projectile_q.get(projectile) else {
        return Ok(());
    };

    let Ok(dmg) = weapon_q.get(cast_weapon.0) else {
        return Ok(());
    };

    if let Ok(enemy) = enemy_q.get(enemy) {
        commands.trigger(EnemyDamageEvent {
            entity_hit: enemy,
            dmg: dmg.0,
            damage_type: DamageType::Physical,
        });

        commands.trigger(EnemyKnockbackEvent {
            entity_hit: enemy,
            projectile,
        });
    }

    commands.entity(projectile).try_despawn();

    Ok(())
}
