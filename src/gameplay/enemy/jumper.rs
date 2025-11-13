use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        enemy::{
            AbilityDamage, AbilitySpeed, DamageCooldown, Enemy, EnemyType, HazardousTerrain, Jump,
            Meele, Owner, Size,
        },
        player::{Direction, Player},
        spells::{Cooldown, Damage, Range, SpellDuration, SpellTick},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(JumperStats {
        health: 10.0,
        damage: 1.0,
        ability_damage: 5.0,
        ability_speed: 200.0,
        range: 400.0,
        cooldown: 4.0,
        size: 60.0,
        sprite: "enemies/jumper.png".to_string(),
    });

    app.add_systems(
        FixedUpdate,
        (move_jumping_jumper).run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(spawn_jumper)
        .add_observer(jumper_attack)
        .add_observer(spawn_jumper_aoe)
        .add_observer(patch_jumper);
}

#[derive(Component)]
#[require(
    EnemyType::Jumper,
    Meele,
    Speed(30.),
    //Meele hit
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    SpellTick(Timer::from_seconds(1.0, TimerMode::Once)),
    SpellDuration(Timer::from_seconds(5.0, TimerMode::Once)),
    Direction(Vec3::ZERO),

)]
pub(crate) struct Jumper;

#[derive(Resource)]
pub(crate) struct JumperStats {
    health: f32,
    damage: f32,
    ability_damage: f32,
    ability_speed: f32,
    range: f32,
    cooldown: f32,
    size: f32,
    sprite: String,
}

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

#[derive(Event)]
pub(crate) struct JumperPatchEvent(pub f32, pub String);

const JUMPER_BUFFER: f32 = 10.0;
const CURVATURE_COEFFICIENT: f32 = 6.0 / 5.0;

fn spawn_jumper(
    _trigger: On<JumperSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_q: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    jumper_q: Query<&Jumper>,
    jumper_stats: Res<JumperStats>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let stats = jumper_stats;

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
            Name::new(format!("Jumper {jumper_count}")),
            Enemy,
            Jumper,
            Sprite {
                image: asset_server.load(stats.sprite.clone()),
                ..default()
            },
            Transform::from_xyz(enemy_pos_x, enemy_pos_y, 10.0)
                .with_scale(Vec3::splat(ENEMY_SIZE / 32.0)),
            Visibility::Visible,
            Health(stats.health),
            Damage(stats.damage),
            AbilityDamage(stats.ability_damage),
            AbilitySpeed(stats.ability_speed),
            Range(stats.range),
            Cooldown(Timer::from_seconds(stats.cooldown, TimerMode::Repeating)),
            Size(stats.size),
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

fn patch_jumper(trigger: On<JumperPatchEvent>, mut stats: ResMut<JumperStats>) {
    let (power_level, sprite) = (trigger.0, &trigger.1);

    stats.health *= power_level;
    stats.damage *= power_level;
    stats.ability_damage *= power_level;
    stats.ability_speed += 50.0 * power_level;
    stats.range += 50.0 * power_level;
    stats.cooldown -= 0.1 * power_level;
    stats.size += 10.0 * power_level;
    stats.sprite = sprite.clone();
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
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let player_pos = player_pos.translation.truncate();

    let jumper = trigger.0;

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
