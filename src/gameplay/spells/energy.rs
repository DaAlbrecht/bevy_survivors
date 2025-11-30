use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

use crate::{
    PausableSystems,
    audio::SfxPool,
    gameplay::{
        damage_numbers::DamageType,
        enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
        player::{Direction, Player},
        simple_animation::{AnimationIndices, AnimationTimer},
        spells::{
            CastSpell, Cooldown, Damage, Halt, PlayerProjectile, ProjectileCount, Spell, SpellType,
            UpgradeSpellEvent,
        },
    },
    screens::Screen,
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Energy,
    Cooldown(Timer::from_seconds(3., TimerMode::Once)),
    Damage(5.),
    ProjectileCount(1.),
    crate::gameplay::Speed(400.),
    Name::new("Energy Spell")
)]
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

    app.add_observer(spawn_energy_projectiles);
    app.add_observer(on_energy_hit);
}

pub fn upgrade_energy(
    _trigger: On<UpgradeSpellEvent>,
    mut energy_q: Query<&mut ProjectileCount, With<Energy>>,
) -> Result {
    let mut count = energy_q.single_mut()?;
    count.0 += 1.0;
    info!("Energy wave count upgraded to: {}", count.0);

    Ok(())
}

fn spawn_energy_projectiles(
    _trigger: On<EnergyAttackEvent>,
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
                    CastSpell(energy),
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
    projectile_q: Query<&CastSpell, With<EnergyProjectile>>,
    spell_q: Query<&Damage, With<Energy>>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;
    let enemy = event.collider2;

    let Ok(cast_spell) = projectile_q.get(projectile) else {
        return Ok(());
    };

    let Ok(dmg) = spell_q.get(cast_spell.0) else {
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
            spell_entity: projectile,
        });
    }

    commands.entity(projectile).try_despawn();

    Ok(())
}
