use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};

use crate::{
    ENEMY_SIZE, GameLayer,
    gameplay::{
        Health, Speed,
        character_controller::CharacterController,
        enemy::{
            AbilityDamage, AbilitySpeed, Charge, Cooldown, DamageCooldown, Enemy, EnemyType, Halt,
            HitDamage, Meele, RANGE_BUFFER, Range, get_valid_spawn_position,
        },
        player::{Direction, Player, PlayerHitEvent},
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
        .add_observer(sprinter_ability_hit)
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
    CollidingEntities::default(),
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
    spatial_q: SpatialQuery,
    rng: Single<&mut WyRand, With<GlobalRng>>,
    sprinter_q: Query<&Sprinter>,
    sprinter_stats: Res<SprinterStats>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let stats = sprinter_stats;

    let Some(enemy_pos) =
        get_valid_spawn_position(spatial_q, player_pos.translation.truncate(), rng)
    else {
        // No valid pos
        return Ok(());
    };

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
        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 0.0)
            .with_scale(Vec3::splat(ENEMY_SIZE / 48.0)),
        CharacterController {
            speed: 30.0,
            ..default()
        },
        Health(stats.health),
        HitDamage(stats.damage),
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
    commands
        .entity(sprinter)
        .insert(Charge)
        //Charge does not collide with enemies whiel charging
        .insert(CollisionLayers::new(
            GameLayer::Enemy,
            [GameLayer::Player, GameLayer::Default],
        ));

    Ok(())
}

fn move_charging_sprinter(
    mut sprinter_q: Query<
        (
            &Transform,
            &mut LinearVelocity,
            Entity,
            &AbilitySpeed,
            &Range,
            &Direction,
            &CollidingEntities,
            &Charge,
        ),
        With<Sprinter>,
    >,
    player_q: Query<&Transform, (With<Player>, Without<Sprinter>)>,
    layer_q: Query<&CollisionLayers>,
    mut commands: Commands,
    time: Res<Time>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let player_pos = player_pos.translation.truncate();

    for (transform, mut linear_velocity, sprinter, speed, range, direction, collisions, _charge) in
        &mut sprinter_q
    {
        let sprinter_pos = transform.translation.truncate();
        let distance = player_pos.distance(sprinter_pos);
        let mut charge_over = false;

        let movement = direction.0 * speed.0 * time.delta_secs();
        linear_velocity.x += movement.x;
        linear_velocity.y += movement.y;

        for colliding_entity in collisions.iter() {
            let Ok(layer) = layer_q.get(*colliding_entity) else {
                return Ok(());
            };

            if layer.memberships.has_all(GameLayer::Default) {
                charge_over = true;
            }

            if layer.memberships.has_all(GameLayer::Player) {
                commands.trigger(SprinterAbilityHitEvent(sprinter));
                charge_over = true;
            }
        }

        if (distance - RANGE_BUFFER) >= range.0 || charge_over {
            commands.entity(sprinter).remove::<Charge>();
            // let charget collide with enemies again
            commands.entity(sprinter).insert(CollisionLayers::new(
                GameLayer::Enemy,
                [GameLayer::Enemy, GameLayer::Player, GameLayer::Default],
            ));
        }
    }

    Ok(())
}

fn sprinter_ability_hit(
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
