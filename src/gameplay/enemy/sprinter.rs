use avian2d::prelude::*;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

use crate::{
    ENEMY_SIZE, SPAWN_RADIUS,
    gameplay::{
        Health, Speed,
        character_controller::CharacterController,
        enemy::{
            AbilityDamage, AbilitySpeed, Charge, DamageCooldown, Enemy, EnemyType, Meele,
            RANGE_BUFFER,
        },
        level::{LevelWalls, find_valid_spawn_position},
        player::{Direction, Player, PlayerHitEvent},
        weapons::{Cooldown, Damage, Halt, Range},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(SprinterStats {
        health: 10.0,
        damage: 1.0,
        ability_damage: 5.0,
        ability_speed: 500.0,
        range: 500.0,
        cooldown: 3.0,
        sprite: "enemies/sprinter.png".to_string(),
    });

    app.add_systems(
        FixedUpdate,
        (move_charging_sprinter).run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(spawn_sprinter)
        .add_observer(sprinter_attack)
        .add_observer(sprinter_abulity_hit)
        .add_observer(patch_sprinter);
}

//"Static Component that do not change from waves"
#[derive(Component)]
#[require(
    EnemyType::Sprinter,
    Meele,
    Speed(50.),
    //Meele hit
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    Direction(Vec3{x:0.,y:0.,z:0.}),
)]
pub(crate) struct Sprinter;

#[derive(Resource)]
pub(crate) struct SprinterStats {
    health: f32,
    damage: f32,
    ability_damage: f32,
    ability_speed: f32,
    range: f32,
    cooldown: f32,
    sprite: String,
}

#[derive(Event)]
pub(crate) struct SprinterAttackEvent(pub Entity);

#[derive(Event)]
pub(crate) struct SprinterAbilityHitEvent(pub Entity);

#[derive(Event)]
pub(crate) struct SprinterSpawnEvent;

#[derive(Event)]
pub(crate) struct SprinterPatchEvent(pub f32, pub String);

fn spawn_sprinter(
    _trigger: On<SprinterSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_q: Query<&Transform, With<Player>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    sprinter_q: Query<&Sprinter>,
    sprinter_stats: Res<SprinterStats>,
    level_walls: Res<LevelWalls>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let stats = sprinter_stats;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    // let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = f32::sin(random_angle);
    let offset_y = SPAWN_RADIUS * f32::cos(random_angle);

    // tile size, search radius
    let desired = Vec2::new(
        player_pos.translation.x + offset_x,
        player_pos.translation.y + offset_y,
    );
    let adjusted_pos = find_valid_spawn_position(desired, &level_walls, 32.0, 8);

    let enemy_pos_x = adjusted_pos.x;
    let enemy_pos_y = adjusted_pos.y;

    let mut sprinter_count = sprinter_q.iter().count();
    sprinter_count += 1;

    commands.spawn((
        Name::new(format!("Shooter {sprinter_count}")),
        Enemy,
        Sprinter,
        Collider::rectangle(32., 32.),
        Sprite {
            image: asset_server.load(stats.sprite.clone()),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 10.0)
            .with_scale(Vec3::splat(ENEMY_SIZE / 48.0)),
        CharacterController {
            speed: 30.0,
            ..default()
        },
        Health(stats.health),
        Damage(stats.damage),
        AbilityDamage(stats.ability_damage),
        AbilitySpeed(stats.ability_speed),
        Range(stats.range),
        Cooldown(Timer::from_seconds(stats.cooldown, TimerMode::Once)),
    ));

    Ok(())
}

fn patch_sprinter(trigger: On<SprinterPatchEvent>, mut stats: ResMut<SprinterStats>) {
    let (power_level, sprite) = (trigger.0, &trigger.1);

    stats.health *= power_level;
    stats.damage *= power_level;
    stats.ability_damage *= power_level;
    stats.ability_speed += 50.0 * power_level;
    stats.range += 50.0 * power_level;
    stats.cooldown -= 0.1 * power_level;
    stats.sprite = sprite.clone();
}

fn sprinter_attack(
    trigger: On<SprinterAttackEvent>,
    mut sprinter_q: Query<(&Transform, &mut Direction, Option<&Halt>), With<Sprinter>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let player_pos = player_pos.translation.truncate();

    let sprinter = trigger.0;

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
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let player_pos = player_pos.translation.truncate();

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
    trigger: On<SprinterAbilityHitEvent>,
    sprinter_q: Query<&AbilityDamage, With<Sprinter>>,
    mut commands: Commands,
) {
    let sprinter = trigger.0;

    let Ok(damage) = sprinter_q.get(sprinter) else {
        return;
    };

    commands.trigger(PlayerHitEvent { dmg: damage.0 });
}
