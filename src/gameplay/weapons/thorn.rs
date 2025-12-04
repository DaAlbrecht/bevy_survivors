use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, CollisionStart, DebugRender, Sensor,
};
use bevy::prelude::*;

use crate::{
    GameLayer, PLAYER_SIZE,
    gameplay::{
        Speed,
        damage_numbers::DamageType,
        enemy::{DamageCooldown, Enemy, EnemyDamageEvent},
        player::{Direction, Player},
        weapons::{
            CastWeapon, Cooldown, Damage, Halt, PlayerProjectile, ProjectileCount, Root,
            UpgradeWeaponEvent, Weapon, WeaponDuration, WeaponType,
            dot::{Bleed, DoT},
        },
    },
    screens::Screen,
};

const THORN_LENGTH: f32 = 16.0;

#[derive(Component)]
#[require(
    Weapon,
    WeaponType::Thorn,
    Segmented,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Once)),
    Speed(600.),
    Damage(1.),
    DoT{
        duration: Timer::from_seconds(5.0, TimerMode::Once),
        tick: Timer::from_seconds(1.0, TimerMode::Once),
        dmg_per_tick: 1.0,
    },
    ProjectileCount(5.0),
    Name::new("Thorn")
)]
#[derive(Reflect)]
pub(crate) struct Thorn;

#[derive(Component, Reflect)]
pub(crate) struct ThornTip;

#[derive(Component, Default, Reflect)]
pub(crate) struct Segmented;

#[derive(Component, Default, Reflect)]
pub(crate) struct ThornSegments(i32);

#[derive(Component, Reflect)]
pub(crate) struct StartPosition(Vec2);

#[derive(Event, Reflect)]
pub(crate) struct ThornAttackEvent;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            thorn_range_keeper.run_if(in_state(Screen::Gameplay)),
            thorn_lifetime.run_if(in_state(Screen::Gameplay)),
        ),
    );
    app.add_observer(spawn_thorn_projectile);
}

pub fn upgrade_thorn(
    _trigger: On<UpgradeWeaponEvent>,
    mut thorn_q: Query<&mut ProjectileCount, With<Thorn>>,
) -> Result {
    let mut count = thorn_q.single_mut()?;
    count.0 += 1.0;
    info!("Thorn projectile count upgraded to: {}", count.0);

    Ok(())
}

fn spawn_thorn_projectile(
    _trigger: On<ThornAttackEvent>,
    player_pos_q: Query<&Transform, With<Player>>,
    thorn_q: Query<Entity, With<Thorn>>,
    enemy_pos_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(player_pos) = player_pos_q.single() else {
        return Ok(());
    };
    let player_pos = player_pos.translation.truncate();

    let thorn = thorn_q.single()?;

    let mut min_distance = f32::MAX;
    let mut clossest_enemy: Option<&Transform> = None;

    for enemy_pos in &enemy_pos_q {
        let distance = player_pos.distance(enemy_pos.translation.truncate());

        if distance < min_distance {
            min_distance = distance;
            clossest_enemy = Some(enemy_pos);
        }
    }

    if let Some(enemy_pos) = clossest_enemy {
        let direction = (enemy_pos.translation.truncate() - player_pos).normalize();
        let thorn_pos = player_pos + (direction * ((PLAYER_SIZE / 2.0) + THORN_LENGTH / 2.0));
        let angle = direction.y.atan2(direction.x);
        commands
            .spawn((
                Name::new("ThornTip"),
                Sprite {
                    image: asset_server.load("fx/thorn_tip.png"),
                    ..default()
                },
                CastWeapon(thorn),
                Transform {
                    translation: Vec3::new(thorn_pos.x, thorn_pos.y, 10.0),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                Direction(direction.extend(10.0)),
                StartPosition(Vec2::new(thorn_pos.x, thorn_pos.y)),
                ThornTip,
                ThornSegments(1),
                PlayerProjectile,
                Sensor,
            ))
            .observe(on_thorn_hit);
    }
    Ok(())
}

fn thorn_range_keeper(
    thorn_q: Query<(Entity, &ProjectileCount), With<Thorn>>,
    mut thorn_tip_q: Query<
        (
            Entity,
            &Transform,
            Option<&Halt>,
            &StartPosition,
            &mut ThornSegments,
        ),
        With<ThornTip>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((thorn, thorn_count)) = thorn_q.single() else {
        return;
    };

    for (thorn_tip, transform, halt, start_pos, mut segments_spawned) in &mut thorn_tip_q {
        let thorn_pos = transform.translation.truncate();
        let distance = thorn_pos.distance(start_pos.0);

        if distance >= segments_spawned.0 as f32 * THORN_LENGTH && halt.is_none() {
            let thorn_base = commands
                .spawn((
                    Name::new("ThornBase"),
                    Sprite {
                        image: asset_server.load("fx/thorn_base.png"),
                        ..default()
                    },
                    CastWeapon(thorn),
                    Transform {
                        translation: Vec3::new(
                            -THORN_LENGTH * (segments_spawned.0) as f32,
                            0.0,
                            10.0,
                        ),
                        rotation: Quat::IDENTITY,
                        ..default()
                    },
                    Collider::rectangle(16., 16.),
                    DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
                    CollisionEventsEnabled,
                    CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
                    Sensor,
                ))
                .observe(on_thorn_hit)
                .id();

            commands.entity(thorn_tip).add_child(thorn_base);
            segments_spawned.0 += 1;
        }

        if distance >= (thorn_count.0 - 1.0) * THORN_LENGTH && halt.is_none() {
            commands.entity(thorn_tip).insert(Halt);

            commands
                .entity(thorn_tip)
                .insert(WeaponDuration(Timer::from_seconds(0.2, TimerMode::Once)));
        }
    }
}

fn thorn_lifetime(
    mut thorn_tip_q: Query<(
        Entity,
        Option<&Children>,
        &mut WeaponDuration,
        &mut ThornSegments,
    )>,
    mut commands: Commands,
) {
    for (thorn_tip, children, mut duration, mut segments) in &mut thorn_tip_q {
        if duration.0.is_finished() {
            let index = (segments.0 - 2) as usize;
            if let Some(child) = children {
                commands.entity(child[index]).despawn();
                segments.0 -= 1;
                duration.0.reset();
            } else {
                commands.entity(thorn_tip).despawn();
            }
        }
    }
}

fn on_thorn_hit(
    event: On<CollisionStart>,
    mut thorn_q: Query<(&Damage, &mut DamageCooldown, &DoT), With<Thorn>>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let enemy = event.collider2;

    let Ok((damage, mut cooldown, dot)) = thorn_q.single_mut() else {
        return Ok(());
    };

    if cooldown.0.is_finished() {
        commands.trigger(EnemyDamageEvent {
            entity_hit: enemy,
            dmg: damage.0,
            damage_type: DamageType::Earth,
        });
        cooldown.0.reset();
    }

    if enemy_q.get(enemy).is_ok() {
        commands
            .entity(enemy)
            .insert_if_new(Root(Timer::from_seconds(0.5, TimerMode::Once)));

        commands.entity(enemy).insert_if_new(Bleed {
            duration: dot.duration.clone(),
            tick: dot.tick.clone(),
            dmg_per_tick: dot.dmg_per_tick,
        });
    }

    Ok(())
}
