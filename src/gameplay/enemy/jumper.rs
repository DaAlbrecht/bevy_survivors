use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE,
    gameplay::{
        Health, Speed,
        enemy::{
            AbilityDamage, AbilitySpeed, DamageCooldown, Enemy, EnemyType, HazardousTerrain, Jump,
            KnockbackDirection, Meele, Owner, SPAWN_RADIUS, Size,
        },
        player::{Direction, Player},
        spells::{Cooldown, Damage, Knockback, Range, SpellDuration, SpellTick},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    // app.add_systems(
    //     Update,
    //     spawn_jumper
    //         .run_if(on_timer(Duration::from_millis(5000)))
    //         .run_if(in_state(Screen::Gameplay))
    //         .in_set(AppSystems::Update),
    // );

    app.add_systems(
        FixedUpdate,
        (move_jumping_jumper).run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(spawn_jumper);
    app.add_observer(jumper_attack);
    app.add_observer(spawn_jumper_aoe);
}

#[derive(Component)]
#[require(
    EnemyType::Jumper,
    Meele,
    Health(10.),
    Speed(30.),
    Knockback(0.0),
    KnockbackDirection(Direction(Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    })),
    //Meele hit
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    //Ability cd
    Cooldown(Timer::from_seconds(4.0,TimerMode::Once)),
    Damage(1.0),
    AbilityDamage(5.0),
    AbilitySpeed(200.0),
    SpellTick(Timer::from_seconds(1.0, TimerMode::Once)),
    SpellDuration(Timer::from_seconds(5.0, TimerMode::Once)),
    Size(60.0),
    Direction(Vec3{x:0.,y:0.,z:0.}),
    Range(500.0),
)]
pub(crate) struct Jumper;

#[derive(Event)]
pub(crate) struct JumperAttackEvent(pub Entity);

#[derive(Event)]
pub(crate) struct JumperLandingEvent(pub Entity);

#[derive(Component)]
pub(crate) struct AbilityVisual;

#[derive(Component)]
pub(crate) struct JumperVisual;

#[derive(Component)]
pub(crate) struct JumperAttackIndicator;

#[derive(Event)]
pub(crate) struct JumperSpawnEvent;

const JUMPER_BUFFER: f32 = 10.0;
const CURVATURE_COEFFICIENT: f32 = 6.0 / 5.0;

fn spawn_jumper(
    _trigger: On<JumperSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    jumper_q: Query<&Jumper>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = SPAWN_RADIUS * f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let mut jumper_count = jumper_q.iter().count();
    jumper_count += 1;

    let jumper = commands
        .spawn((
            Name::new(format!("Shooter {jumper_count}")),
            Enemy,
            Jumper,
            Sprite {
                image: asset_server.load("enemies/jumper.png"),
                ..default()
            },
            Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
            Visibility::Visible,
        ))
        .id();

    let shadow = commands
        .spawn((
            Name::new(format!("Shadow {jumper_count}")),
            AbilityVisual,
            Sprite {
                image: asset_server.load("enemies/shadow.png"),
                ..default()
            },
            Visibility::Hidden,
            Transform::from_xyz(0.0, -ENEMY_SIZE / 2.0, -1.0),
        ))
        .id();

    let jumper_visual = commands
        .spawn((
            Name::new(format!("Jumper_Visual{jumper_count}")),
            AbilityVisual,
            JumperVisual,
            Sprite {
                image: asset_server.load("enemies/jumper.png"),
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();

    commands.entity(jumper).add_child(shadow);
    commands.entity(jumper).add_child(jumper_visual);

    Ok(())
}

fn jumper_attack(
    trigger: On<JumperAttackEvent>,
    mut jumper_q: Query<
        (&Transform, &mut Direction, &mut Visibility, &Children),
        (With<Jumper>, Without<JumperVisual>),
    >,
    mut visual_q: Query<&mut Visibility, (With<AbilityVisual>, Without<Jumper>)>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let jumper = trigger.0;
    let player_pos = player_q.single()?.translation.truncate();

    let Ok((transform, mut direction, mut visibility, children)) = jumper_q.get_mut(jumper) else {
        return Ok(());
    };

    let jumper_pos = transform.translation.truncate();
    let target_offset = Vec2::new(
        rand::random_range(-7.5..=7.5) as f32,
        rand::random_range(-7.5..=7.5) as f32,
    );

    let target_pos = player_pos + target_offset;

    direction.0 = (target_pos - jumper_pos).normalize().extend(0.0);

    commands.entity(jumper).insert(Jump {
        start_pos: transform.translation.truncate(),
        target_pos,
    });

    commands.spawn((
        Sprite {
            image: asset_server.load("enemies/jumper_aoe_indicator.png"),
            ..default()
        },
        Transform {
            translation: target_pos.extend(-1.0),
            ..default()
        },
        JumperAttackIndicator,
        Owner(jumper),
    ));

    //Hide Jumper Sprite
    *visibility = Visibility::Hidden;

    //Make Jumper_Viusal and Shadow Visual visibil
    for &child in children {
        let Ok(mut child_visibility) = visual_q.get_mut(child) else {
            continue;
        };
        *child_visibility = Visibility::Visible;
    }

    Ok(())
}

fn move_jumping_jumper(
    mut jumper_q: Query<
        (
            &mut Transform,
            Entity,
            &AbilitySpeed,
            &Direction,
            Option<&Jump>,
            &Children,
            &mut Visibility,
        ),
        (With<Jumper>, Without<AbilityVisual>),
    >,
    mut visual_q: Query<
        (&mut Transform, &mut Visibility, Option<&JumperVisual>),
        (With<AbilityVisual>, Without<Jumper>),
    >,
    mut commands: Commands,
    time: Res<Time>,
) -> Result {
    for (mut transform, jumper, speed, direction, jump, children, mut visibility) in &mut jumper_q {
        let jumper_pos = transform.translation.truncate();

        if let Some(jump) = jump {
            let distance = jumper_pos.distance(jump.target_pos);
            let radius = jump.target_pos.distance(jump.start_pos);

            //Move jumper enity and shadow
            let movement = direction.0 * speed.0 * time.delta_secs();
            transform.translation += movement;

            //Jumper Visual
            for &child in children {
                let Ok((mut child_transform, mut child_visibility, jumper_visual)) =
                    visual_q.get_mut(child)
                else {
                    continue;
                };

                if jumper_visual.is_some() {
                    let jumped_distance = jump.start_pos.distance(jumper_pos);

                    let jump_hight = CURVATURE_COEFFICIENT
                        * jumped_distance
                        * (1.0 - (jumped_distance / radius));

                    child_transform.translation.y = jump_hight;
                }

                if distance <= JUMPER_BUFFER {
                    *child_visibility = Visibility::Hidden;
                    *visibility = Visibility::Visible;
                }
            }

            // If we are close to player
            if distance <= JUMPER_BUFFER {
                commands.entity(jumper).remove::<Jump>();
                commands.trigger(JumperLandingEvent(jumper));
            }
        }
    }

    Ok(())
}

fn spawn_jumper_aoe(
    trigger: On<JumperLandingEvent>,
    jumper_q: Query<
        (
            &Transform,
            &AbilityDamage,
            &SpellDuration,
            &SpellTick,
            &Size,
        ),
        With<Jumper>,
    >,
    indicator_q: Query<(Entity, &Owner), With<JumperAttackIndicator>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let jumper = trigger.0;
    let Ok((transform, damage, duration, ticker, size)) = jumper_q.get(jumper) else {
        return;
    };
    let jumper_pos = transform.translation.truncate();

    for (indicator, owner) in indicator_q {
        if owner.0 == jumper {
            commands.entity(indicator).despawn();
        }
    }

    commands.spawn((
        Sprite {
            image: asset_server.load("enemies/jumper_aoe.png"),
            ..default()
        },
        Transform {
            translation: jumper_pos.extend(-1.0),
            ..default()
        },
        HazardousTerrain,
        Damage(damage.0),
        SpellDuration(duration.0.clone()),
        SpellTick(ticker.0.clone()),
        Size(size.0),
    ));
}
