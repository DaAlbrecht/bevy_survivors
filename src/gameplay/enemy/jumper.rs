use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    AppSystems, ENEMY_SIZE,
    gameplay::{
        Health,
        enemy::{
            AbilityDamage, AbilitySpeed, DamageCooldown, Enemy, EnemyType, Jump,
            KnockbackDirection, Meele, SPAWN_RADIUS, Speed,
        },
        player::{Direction, Player},
        spells::{Cooldown, Damage, Knockback, Range},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_jumper
            .run_if(on_timer(Duration::from_millis(5000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );

    app.add_systems(
        FixedUpdate,
        (move_jumping_jumper).run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(jumper_attack);
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
    AbilitySpeed(350.0),
    Direction(Vec3{x:0.,y:0.,z:0.}),
    Range(500.0),
)]
pub(crate) struct Jumper;

#[derive(Event)]
pub(crate) struct JumperAttackEvent(pub Entity);

#[derive(Component)]
pub(crate) struct AbilityVisual;

#[derive(Component)]
pub(crate) struct JumperVisual;

const JUMPER_BUFFER: f32 = 10.0;
const CURVATURE_COEFFICIENT: f32 = 6.0 / 5.0;

fn spawn_jumper(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
    jumper_q: Query<&Jumper>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

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
    trigger: Trigger<JumperAttackEvent>,
    mut jumper_q: Query<
        (&Transform, &mut Direction, &mut Visibility, &Children),
        (With<Jumper>, Without<JumperVisual>),
    >,
    mut visual_q: Query<&mut Visibility, (With<AbilityVisual>, Without<Jumper>)>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let jumper = trigger.0;
    let player_pos = player_q.single()?.translation.truncate();

    let Ok((transform, mut direction, mut visibility, children)) = jumper_q.get_mut(jumper) else {
        return Ok(());
    };

    let jumper_pos = transform.translation.truncate();
    let target_offset = Vec2::new(
        rand::random_range(0..=15) as f32,
        rand::random_range(0..=15) as f32,
    );

    let target_pos = player_pos + target_offset;

    direction.0 = (target_pos - jumper_pos).normalize().extend(0.0);

    commands.entity(jumper).insert(Jump {
        start_pos: transform.translation.truncate(),
        target_pos,
    });

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
            }
        }
    }

    Ok(())
}
