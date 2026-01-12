use avian2d::prelude::*;
use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

use crate::{
    GameLayer, PLAYER_SIZE, PROJECTILE_SIZE, PausableSystems, PostPhysicsAppSystems,
    gameplay::{
        Despawn, Speed,
        character_controller::CharacterController,
        enemy::{
            jumper::{JumperAttackEvent, JumperAttackIndicator},
            shooter::{ShooterAttackEvent, ShooterProjectileHitEvent},
            sprinter::SprinterAttackEvent,
        },
        player::{Direction, PlayerHitEvent},
    },
    screens::Screen,
};

use super::player::Player;

pub(crate) mod damage;
pub(crate) mod jumper;
pub(crate) mod shooter;
pub(crate) mod sprinter;
pub(crate) mod walker;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        damage::plugin,
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
}

const RANGE_BUFFER: f32 = 50.0;

#[derive(Component, Default, Reflect)]
pub(crate) struct DamageCooldown(pub Timer);

#[derive(Component, Default, Reflect)]
#[require(
    DespawnOnExit::<Screen>(Screen::Gameplay),
    Direction,
    LockedAxes::ROTATION_LOCKED,
    RigidBody::Dynamic,
    Collider = Collider::circle(16.),
    Friction = Friction::ZERO,
    CollisionLayers = CollisionLayers::new(GameLayer::Enemy,[
    GameLayer::Enemy,
    GameLayer::Player,
]))]
pub(crate) struct Enemy;

#[derive(Event, Reflect)]
pub(crate) struct PlayerPushingEvent(pub Entity);

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

#[derive(Component, Default, Reflect)]
pub(crate) struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Range(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Root(pub Timer);

#[derive(Component)]
pub(crate) struct Halt;

#[derive(Component)]
pub(crate) struct HitDamage(pub f32);

#[derive(Component)]
pub(crate) struct AbilityDamage(pub f32);

#[derive(Component)]
pub(crate) struct AbilitySpeed(pub f32);

#[derive(Component)]
pub(crate) struct AbilityDuration(pub Timer);

#[derive(Component)]
pub(crate) struct AbilityTick(pub Timer);

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
    mut enemy_q: Query<
        (
            &CharacterController,
            &Transform,
            &mut LinearVelocity,
            &mut Direction,
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
    let player_pos = player_pos.translation;

    for (
        controller,
        transform,
        mut linear_velocity,
        mut intended_direction,
        root,
        halt,
        charge,
        jump,
    ) in &mut enemy_q
    {
        if root.is_some() || halt.is_some() {
            //skip movement if enemy gets knockedback or is rooted
            linear_velocity.x = 0.;
            linear_velocity.y = 0.;
            intended_direction.0 = Vec3::ZERO;
        } else if jump.is_some() || charge.is_some() {
            continue;
        } else {
            let enemy_pos = transform.translation;
            let to_player = player_pos - enemy_pos;
            if to_player.length_squared() <= 0.0001 {
                linear_velocity.x = 0.;
                linear_velocity.y = 0.;
                intended_direction.0 = Vec3::ZERO;
                continue;
            }
            let direction = to_player.normalize();
            intended_direction.0 = direction;

            let desired = direction * controller.speed;
            linear_velocity.x = linear_velocity.x + (desired.x - linear_velocity.x) * 0.15;
            linear_velocity.y = linear_velocity.y + (desired.y - linear_velocity.y) * 0.15;
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

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(mut enemies_q: Query<(&Direction, &mut Sprite), With<Enemy>>) {
    for (intended_direction, mut sprite) in &mut enemies_q {
        let dx = intended_direction.0.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }
    }
}

fn enemy_despawner(
    enemy_q: Query<Entity, (With<Enemy>, With<Despawn>)>,
    indicator_q: Query<(Entity, &Owner), With<JumperAttackIndicator>>,
    mut commands: Commands,
) {
    for enemy in &enemy_q {
        //We can prop solve this better via relations
        for (indicator, owner) in indicator_q {
            if owner.0 == enemy {
                commands.entity(indicator).despawn();
            }
        }
        commands.entity(enemy).despawn_children();
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
            if (player_pos.translation.distance(projectile_pos.translation)
                - (PROJECTILE_SIZE / 2.0))
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
            &HitDamage,
            &mut AbilityDuration,
            &mut AbilityTick,
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
