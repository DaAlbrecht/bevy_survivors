use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystems,
    gameplay::{
        Health,
        enemy::{
            AbilityDamage, AbilitySpeed, Charge, DamageCooldown, Enemy, EnemyType,
            KnockbackDirection, RANGE_BUFFER, SPAWN_RADIUS, Speed,
        },
        player::{Direction, Player, PlayerHitEvent},
        spells::{Cooldown, Damage, Halt, Knockback, Range},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_sprinter
            .run_if(on_timer(Duration::from_millis(5000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );

    app.add_systems(
        FixedUpdate,
        (move_charging_sprinter).run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(sprinter_attack);
    app.add_observer(sprinter_abulity_hit);
}

#[derive(Component)]
#[require(
    EnemyType::Sprinter,
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
    //Ability cd
    Cooldown(Timer::from_seconds(3.0,TimerMode::Once)),
    Damage(1.0),
    AbilityDamage(5.0),
    AbilitySpeed(500.0),
    Direction(Vec3{x:0.,y:0.,z:0.}),
    Range(250.0),
)]
pub(crate) struct Sprinter;

#[derive(Event)]
pub(crate) struct SprinterAttackEvent(pub Entity);

#[derive(Event)]
pub(crate) struct SprinterAbilityHitEvent(pub Entity);

fn spawn_sprinter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
    sprinter_q: Query<&Sprinter>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let mut sprinter_count = sprinter_q.iter().count();
    sprinter_count += 1;

    commands.spawn((
        Name::new(format!("Shooter {sprinter_count}")),
        Enemy,
        Sprinter,
        Sprite {
            image: asset_server.load("enemies/Sprinter.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
    ));

    Ok(())
}

fn sprinter_attack(
    trigger: Trigger<SprinterAttackEvent>,
    mut sprinter_q: Query<(&Transform, &mut Direction, Option<&Halt>), With<Sprinter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let sprinter = trigger.0;
    let player_pos = player_q.single()?.translation.truncate();

    let Ok((transform, mut direction, halt)) = sprinter_q.get_mut(sprinter) else {
        return Ok(());
    };

    let sprinter_pos = transform.translation.truncate();
    direction.0 = (player_pos - sprinter_pos).normalize().extend(0.0);

    if halt.is_some() {
        commands.entity(sprinter).remove::<Halt>();
    }
    commands.entity(sprinter).insert(Charge {
        active: true,
        hit_target: false,
    });
    Ok(())
}

fn move_charging_sprinter(
    mut sprinter_q: Query<
        (
            &mut Transform,
            Entity,
            &AbilitySpeed,
            &Range,
            &Direction,
            Option<&Charge>,
        ),
        With<Sprinter>,
    >,
    player_q: Query<&Transform, (With<Player>, Without<Sprinter>)>,
    mut commands: Commands,
    time: Res<Time>,
) -> Result {
    let player_pos = player_q.single()?.translation.truncate();

    for (mut transform, sprinter, speed, range, direction, charge) in &mut sprinter_q {
        let sprinter_pos = transform.translation.truncate();
        let distance = player_pos.distance(sprinter_pos);
        if charge.is_some() {
            let movement = direction.0 * speed.0 * time.delta_secs();
            transform.translation += movement;
            if (distance - RANGE_BUFFER) >= range.0 {
                commands.entity(sprinter).remove::<Charge>();
            }
        }
    }

    Ok(())
}

fn sprinter_abulity_hit(
    trigger: Trigger<SprinterAbilityHitEvent>,
    sprinter_q: Query<&AbilityDamage, With<Sprinter>>,
    mut commands: Commands,
) {
    let sprinter = trigger.0;

    let Ok(damage) = sprinter_q.get(sprinter) else {
        return;
    };

    commands.trigger(PlayerHitEvent { dmg: damage.0 });
}
