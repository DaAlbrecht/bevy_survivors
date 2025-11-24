use avian2d::prelude::*;
use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};
use rand::Rng;

use crate::{
    GameLayer, PLAYER_SIZE, PausableSystems, PostPhysicsAppSystems, SPELL_SIZE,
    gameplay::{
        Health, Speed,
        character_controller::CharacterController,
        damage_numbers::DamageMessage,
        enemy::{
            jumper::JumperAttackEvent,
            shooter::{ShooterAttackEvent, ShooterProjectileHitEvent},
            sprinter::SprinterAttackEvent,
        },
        player::{Direction, PlayerHitEvent},
        simple_animation::HurtAnimationTimer,
        spells::{Cooldown, Damage, Despawn, Halt, Range, Root, SpellDuration, SpellTick},
    },
    screens::Screen,
};

use super::player::Player;

pub(crate) mod jumper;
pub(crate) mod shooter;
pub(crate) mod sprinter;
pub(crate) mod walker;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        jumper::plugin,
        walker::plugin,
        shooter::plugin,
        sprinter::plugin,
    ));

    app.add_systems(
        Update,
        ((update_animation_movement,)
            .chain()
            .in_set(PostPhysicsAppSystems::PlayAnimations),)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_systems(
        FixedUpdate,
        (
            enemy_timer_handle,
            enemy_movement,
            projectile_hit_detection,
            attack,
            enemy_range_keeper,
            terrain_manager,
            move_enemy_projectile,
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_systems(
        FixedLast,
        ((enemy_despawner).in_set(PostPhysicsAppSystems::Update))
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_observer(enemy_take_dmg);
}

const RANGE_BUFFER: f32 = 50.0;

#[derive(Component, Default, Reflect)]
pub(crate) struct DamageCooldown(pub Timer);

#[derive(Component, Default, Reflect)]
#[require(
    LockedAxes::ROTATION_LOCKED,
    RigidBody::Dynamic,
    Collider = Collider::rectangle(32., 32.),
    CollisionLayers = CollisionLayers::new(GameLayer::Enemy,[
    GameLayer::Enemy,
    GameLayer::Player,
    GameLayer::Default,
    GameLayer::PlayerProjectiles,
]))]
pub(crate) struct Enemy;

#[derive(Event, Reflect)]
pub(crate) struct PlayerPushingEvent(pub Entity);

#[derive(Event, Reflect)]
pub(crate) struct EnemyDamageEvent {
    pub entity_hit: Entity,
    pub dmg: f32,
}

#[derive(Event, Reflect)]
pub(crate) struct EnemyKnockbackEvent {
    pub entity_hit: Entity,
    pub spell_entity: Entity,
}

#[derive(Event, Reflect)]
pub(crate) struct EnemyDeathEvent(pub Transform);

#[derive(Component, Reflect)]
pub(crate) struct Colliding;

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    Collider = Collider::rectangle(16., 16.),
    DebugRender = DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
    CollisionLayers = CollisionLayers::new(GameLayer::Enemy,[
    GameLayer::Player,
    GameLayer::Default,
]))]
pub(crate) struct EnemyProjectile;

#[derive(Component)]
#[relationship(relationship_target = EnemyProjectiles)]
#[derive(Reflect)]
pub(crate) struct ProjectileOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ProjectileOf, linked_spawn)]
#[derive(Reflect)]
pub(crate) struct EnemyProjectiles(Vec<Entity>);

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub(crate) enum EnemyType {
    Walker,
    Shooter,
    Sprinter,
    Jumper,
    None,
}

#[derive(Component)]
pub(crate) struct AbilityDamage(pub f32);

#[derive(Component)]
pub(crate) struct AbilitySpeed(pub f32);

#[derive(Component)]
#[allow(dead_code)]
pub(crate) struct Charge {
    active: bool,
    hit_target: bool,
}

#[derive(Component)]
pub(crate) struct Jump {
    start_pos: Vec2,
    target_pos: Vec2,
}

#[derive(Component)]
pub(crate) struct Owner(pub Entity);

#[derive(Component, Default)]
pub(crate) struct Ranged;

#[derive(Component, Default)]
pub(crate) struct Meele;

#[derive(Component)]
pub(crate) struct HazardousTerrain;

#[derive(Component)]
pub(crate) struct Size(pub f32);

//BUG: Enemies with Halt get dragged by the player refactor collison handle and halt
fn enemy_movement(
    enemy_q: Query<
        (
            &CharacterController,
            &Transform,
            &mut LinearVelocity,
            Option<&Root>,
            Option<&Halt>,
            Option<&Charge>,
            Option<&Jump>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    let Ok(player_pos) = player_q.single() else {
        return;
    };
    let player_pos = player_pos.translation.truncate();

    for (controller, transform, mut linear_velocity, root, halt, charge, jump) in enemy_q {
        if root.is_some() || halt.is_some() || charge.is_some() || jump.is_some() {
            //skip movement if enemy gets knockedback or is rooted
            linear_velocity.x = 0.;
            linear_velocity.y = 0.;
        } else {
            let enemy_pos = transform.translation.truncate();
            let to_player = player_pos - enemy_pos;
            if to_player.length_squared() <= 0.0001 {
                linear_velocity.x = 0.;
                linear_velocity.y = 0.;
                continue;
            }
            let velocity = to_player.normalize() * controller.speed;
            linear_velocity.x = velocity.x;
            linear_velocity.y = velocity.y;
        }
    }
}

fn move_enemy_projectile(
    enemy_q: Query<Entity, With<Enemy>>,
    projectiles: Query<&EnemyProjectiles>,
    mut projectile_q: Query<
        (&mut LinearVelocity, &Direction, &Speed),
        (With<EnemyProjectile>, Without<Halt>),
    >,
) {
    //Loop over all types of enemies
    for enemy in &enemy_q {
        // Iter over each projectile for this given enemy type
        for projectile in projectiles.iter_descendants(enemy) {
            let Ok((mut linear_velocity, direction, speed)) = projectile_q.get_mut(projectile)
            else {
                continue;
            };

            let movement = direction.0 * speed.0;
            linear_velocity.x = movement.x;
            linear_velocity.y = movement.y;
        }
    }
}

///
/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(mut enemies_q: Query<(&LinearVelocity, &mut Sprite), With<Enemy>>) {
    for (velocity, mut sprite) in &mut enemies_q {
        let dx = velocity.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }
    }
}

//maybe refactor with timer handle later?
//this is where the player get damaged form touching an enemy
fn attack(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_dmg_timer_q: Query<(&mut DamageCooldown, &Damage), (With<Enemy>, With<Colliding>)>,
) {
    for (mut timer, damage) in &mut enemy_dmg_timer_q {
        if timer.0.tick(time.delta()).just_finished() {
            commands.trigger(PlayerHitEvent { dmg: damage.0 });
        }
    }
}

fn enemy_take_dmg(
    trigger: On<EnemyDamageEvent>,
    mut damage_writer: MessageWriter<DamageMessage>,
    mut enemy_q: Query<(&mut Health, &Transform), (With<Enemy>, Without<Despawn>)>,
    mut commands: Commands,
) {
    let enemy_entity = trigger.entity_hit;

    commands
        .entity(enemy_entity)
        .insert(HurtAnimationTimer::default());

    if let Ok((mut health, transform)) = enemy_q.get_mut(enemy_entity) {
        health.0 -= trigger.dmg;

        //TODO: GET REAL CRIT
        let mut rng = rand::rng();
        let is_crit = rng.random_bool(0.10);
        damage_writer.write(DamageMessage {
            amount: trigger.dmg as i32,
            world_pos: transform.translation.truncate(),
            crit: is_crit,
        });

        if health.0 <= 0.0 {
            commands.trigger(EnemyDeathEvent(*transform));
            commands.entity(enemy_entity).insert(Despawn);
        }
    }
}

//fn enemy_get_pushed_from_hit(
//    trigger: On<EnemyKnockbackEvent>,
//    mut enemy_q: Query<
//        (&mut LinearVelocity, Option<&Charge>),
//        (With<Enemy>, Without<PlayerProjectile>),
//    >,
//    projectile_q: Query<&LinearVelocity, (With<PlayerProjectile>, Without<Enemy>)>,
//) -> Result {
//    let enemy_entity = trigger.entity_hit;
//    let projectile_entity = trigger.spell_entity;
//    let projectile_mc = projectile_q.get(projectile_entity)?;
//
//    projectile_mc.as_dvec2() * pro
//
//    let proj_world_vel = projectile_mc.velocity * projectile_mc.speed;
//    if proj_world_vel.length_squared() <= 1e-6 {
//        return Ok(());
//    }
//
//    let dir = proj_world_vel.normalize();
//
//    if let Ok((mut enemy_move, charge)) = enemy_q.get_mut(enemy_entity) {
//        if charge.is_some() {
//            // Charging enemies cannot be knocked back
//            return Ok(());
//        }
//
//        enemy_move.apply_knockback_from_source(dir, projectile_mc);
//    }
//
//    Ok(())
//}

fn enemy_despawner(enemy_q: Query<Entity, (With<Enemy>, With<Despawn>)>, mut commands: Commands) {
    for enemy in &enemy_q {
        commands.entity(enemy).despawn();
    }
}

//Inserts Halt if enemy is in range to player and is ranged
fn enemy_range_keeper(
    enemy_q: Query<(Entity, &Transform, &Range, Option<&Halt>), (With<Enemy>, With<Ranged>)>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let player_pos = player_pos.translation.truncate();

    for (enemy, enemy_transform, range, halt) in &enemy_q {
        let enemy_pos = enemy_transform.translation.truncate();
        let distance = enemy_pos.distance(player_pos);

        if distance < range.0 && halt.is_none() {
            if enemy.is_empty() {
                continue;
            }
            commands.entity(enemy).insert(Halt);
        } else if distance > (RANGE_BUFFER + range.0) && halt.is_some() {
            if enemy.is_empty() {
                continue;
            }
            commands.entity(enemy).remove::<Halt>();
        }
    }

    Ok(())
}

fn enemy_timer_handle(
    mut cooldown_q: Query<
        (
            Entity,
            &mut Cooldown,
            &EnemyType,
            &Transform,
            Option<&Halt>,
            Option<&Range>,
        ),
        With<Enemy>,
    >,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };
    let player_pos = player_pos.translation;

    for (enemy, mut cooldown_timer, enemy_type, transform, halt, range) in &mut cooldown_q {
        cooldown_timer.0.tick(time.delta());

        if cooldown_timer.0.is_finished() {
            match enemy_type {
                EnemyType::Shooter => {
                    if halt.is_some() {
                        commands.trigger(ShooterAttackEvent(enemy));
                    }
                }
                //We calculate only in the case so we dont cluter the update loop with unneeded calculations
                EnemyType::Sprinter => {
                    let distance = player_pos.distance(transform.translation);
                    if let Some(range) = range
                        && range.0 >= distance
                    {
                        commands.trigger(SprinterAttackEvent(enemy));
                    }
                }
                //We calculate only in the case so we dont cluter the update loop with unneeded calculations
                EnemyType::Jumper => {
                    let distance = player_pos.distance(transform.translation);
                    if let Some(range) = range
                        && range.0 >= distance
                    {
                        commands.trigger(JumperAttackEvent(enemy));
                    }
                }
                _ => (),
            }

            cooldown_timer.0.reset();
        }
    }

    Ok(())
}

fn projectile_hit_detection(
    enemy_q: Query<(Entity, &EnemyType), With<Enemy>>,
    player_q: Query<&Transform, With<Player>>,
    projectiles: Query<&EnemyProjectiles>,
    projectile_q: Query<&Transform, With<EnemyProjectile>>,
    mut commands: Commands,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    // Get all enemies
    for (enemy, enemy_type) in &enemy_q {
        // Get each projectile of this enemy
        for projectile in projectiles.iter_descendants(enemy) {
            // Get position of this particular projectile
            let projectile_pos = projectile_q.get(projectile)?;

            //Check if player is hit by this projectile
            if (player_pos.translation.distance(projectile_pos.translation) - (SPELL_SIZE / 2.0))
                <= (PLAYER_SIZE / 2.0)
            {
                trigger_player_hit_event(enemy_type, projectile, enemy, &mut commands);
            }
        }
    }

    Ok(())
}

fn trigger_player_hit_event(
    enemy_type: &EnemyType,
    projectile: Entity,
    enemy: Entity,
    commands: &mut Commands,
) {
    if let EnemyType::Shooter = enemy_type {
        commands.trigger(ShooterProjectileHitEvent {
            projectile,
            source: enemy,
        });
    }
}

//Handles terrain collision lifetime damge etc
//TODO: Fix Physics collision detection with terrain
fn terrain_manager(
    mut terrain_q: Query<
        (
            Entity,
            &Transform,
            &Damage,
            &mut SpellDuration,
            &mut SpellTick,
            &Size,
        ),
        With<HazardousTerrain>,
    >,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let player_pos = player_pos.translation.truncate();

    for (terrain, transform, damage, mut duration, mut ticker, size) in &mut terrain_q {
        let terrain_pos = transform.translation.truncate();
        let distance = terrain_pos.distance(player_pos);

        duration.0.tick(time.delta());
        ticker.0.tick(time.delta());

        if ticker.0.is_finished() && distance <= size.0 {
            commands.trigger(PlayerHitEvent { dmg: damage.0 });
            info!("Terrain_dmg");
            ticker.0.reset();
        }

        if duration.0.is_finished() {
            commands.entity(terrain).despawn();
        }
    }

    Ok(())
}
