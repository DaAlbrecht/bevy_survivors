use std::{f32::consts::PI, time::Duration};

use bevy::{
    ecs::relationship::RelationshipSourceCollection, prelude::*, time::common_conditions::on_timer,
};

use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystems,
    gameplay::{
        Health,
        enemy::{
            DamageCooldown, Enemy, EnemyProjectile, EnemyType, KnockbackDirection, ProjectileOf,
            ProjectileSpeed, SPAWN_RADIUS, Speed, separation_force_calc,
        },
        player::{Direction, Player, PlayerHitEvent},
        spells::{Cooldown, Damage, Halt, Knockback, Range, Root},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_shooter
            .run_if(on_timer(Duration::from_millis(2000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
    app.add_systems(
        FixedUpdate,
        (
            shooter_movement,
            shooter_range_keeper,
            // move_shooter_projectiles,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(shooter_attack);
    app.add_observer(shooter_projectile_hit);

    // app.add_systems(Update, (walker_movement).run_if(in_state(Screen::Gameplay)));
}

const RANGE_BUFFER: f32 = 50.0;

#[derive(Component)]
#[require(
    EnemyType::Shooter,
    Health(10.),
    Speed(100.),
    Knockback(0.0),
    KnockbackDirection(Direction(Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    })),
    //Meele hit
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    //Shoot cd
    Cooldown(Timer::from_seconds(2.0,TimerMode::Once)),
    Damage(5.0),
    Range(200.0),
    ProjectileSpeed(125.),
)]
pub(crate) struct Shooter;

#[derive(Event)]
pub(crate) struct ShooterAttackEvent(pub Entity);

#[derive(Event)]
pub(crate) struct ShooterProjectileHitEvent {
    pub projectile: Entity,
    pub source: Entity,
}

fn spawn_shooter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
    shooter_q: Query<&Shooter>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let mut shooter_count = shooter_q.iter().count();
    shooter_count += 1;

    commands.spawn((
        Name::new(format!("Shooter {shooter_count}")),
        Enemy,
        Shooter,
        Sprite {
            image: asset_server.load("enemies/Shooter.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
    ));

    Ok(())
}

fn shooter_movement(
    shooter_q: Query<
        (
            &mut Transform,
            &Speed,
            &Knockback,
            Option<&Root>,
            Option<&Halt>,
        ),
        With<Shooter>,
    >,
    player_q: Query<&Transform, (With<Player>, Without<Shooter>)>,
    time: Res<Time>,
) -> Result {
    let player_pos = player_q.single()?.translation.truncate();

    let shooter_positions = shooter_q
        .iter()
        .map(|t| t.0.translation.truncate())
        .collect::<Vec<Vec2>>();

    for (mut transform, speed, knockback, root, halt) in shooter_q {
        let shoter_pos = transform.translation.truncate();
        if knockback.0 > 1.0 || root.is_some() || halt.is_some() {
            //skip movement if enemy gets knockedback or is rooted
            continue;
        }

        let direction = (player_pos - shoter_pos).normalize();

        let separation_force = separation_force_calc(&shooter_positions, shoter_pos, player_pos);

        let movement = (direction + separation_force).normalize() * (speed.0 * time.delta_secs());
        transform.translation += movement.extend(0.0);
    }

    Ok(())
}

fn shooter_range_keeper(
    shooter_q: Query<(Entity, &Transform, &Range, Option<&Halt>), With<Shooter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_q.single()?.translation.truncate();

    for (shooter, transform, range, halt) in &shooter_q {
        let shooter_pos = transform.translation.truncate();
        let distance = shooter_pos.distance(player_pos);

        if distance < range.0 && halt.is_none() {
            if shooter.is_empty() {
                continue;
            }
            info!("inserting halt");

            commands.entity(shooter).insert(Halt);
        } else if distance > (RANGE_BUFFER + range.0) && halt.is_some() {
            if shooter.is_empty() {
                continue;
            }
            info!("removing halt");

            commands.entity(shooter).remove::<Halt>();
        }
    }

    Ok(())
}

fn shooter_attack(
    trigger: Trigger<ShooterAttackEvent>,
    shooter_q: Query<&Transform, With<Shooter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let shooter = trigger.0;
    let player_pos = player_q.single()?.translation.truncate();

    let Ok(transform) = shooter_q.get(shooter) else {
        return Ok(());
    };

    let shooter_pos = transform.translation.truncate();
    let direction = (player_pos - shooter_pos).normalize();
    let angle = direction.y.atan2(direction.x);

    commands.spawn((
        Sprite {
            image: asset_server.load("enemies/shooter_bullet.png"),
            ..default()
        },
        Transform {
            translation: transform.translation,
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        EnemyProjectile,
        ProjectileOf(shooter),
        Direction(direction.extend(0.0)),
    ));

    Ok(())
}

fn shooter_projectile_hit(
    trigger: Trigger<ShooterProjectileHitEvent>,
    shooter_q: Query<&Damage, With<Shooter>>,
    mut commands: Commands,
) {
    info!("hit player");
    let projectile = trigger.projectile;
    let shooter = trigger.source;

    let Ok(damage) = shooter_q.get(shooter) else {
        return;
    };

    commands.trigger(PlayerHitEvent { dmg: damage.0 });

    commands.entity(projectile).despawn();
}
