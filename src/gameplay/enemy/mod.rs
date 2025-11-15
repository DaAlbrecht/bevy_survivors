use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

use crate::{
    PLAYER_SIZE, PausableSystems, PhysicsAppSystems, SPELL_SIZE,
    gameplay::{
        Health,
        enemy::{
            jumper::JumperAttackEvent,
            shooter::{ShooterAttackEvent, ShooterProjectileHitEvent},
            sprinter::{SprinterAbilityHitEvent, SprinterAttackEvent},
        },
        movement::{MovementController, PhysicalTranslation},
        player::PlayerHitEvent,
        spells::{
            Cooldown, Damage, Despawn, Halt, PlayerProjectile, Range, Root, SpellDuration,
            SpellTick,
        },
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
        FixedUpdate,
        (
            (enemy_timer_handle),
            (enemy_movement).in_set(PhysicsAppSystems::PhysicsAdjustments),
            (
                enemy_colliding_detection,
                enemy_stop_colliding_detection,
                enemy_push_detection,
                projectile_hit_detection,
                attack,
                enemy_range_keeper,
                terrain_manager,
            )
                .in_set(PhysicsAppSystems::PhysicsResolution),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_systems(
        FixedLast,
        ((enemy_despawner).in_set(PhysicsAppSystems::PhysicsAdjustments))
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_observer(enemy_pushing)
        .add_observer(enemy_take_dmg)
        .add_observer(enemy_get_pushed_from_hit);
}

const SEPARATION_RADIUS: f32 = 40.;
const SEPARATION_FORCE: f32 = 10.;
const RANGE_BUFFER: f32 = 50.0;
const PUSH_FORCE: f32 = 20.0;

#[derive(Component, Default, Reflect)]
pub(crate) struct DamageCooldown(pub Timer);

#[derive(Component, Default, Reflect)]
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
            &mut MovementController,
            &PhysicalTranslation,
            Option<&Root>,
            Option<&Halt>,
            Option<&Charge>,
            Option<&Jump>,
        ),
        With<Enemy>,
    >,
    player_q: Query<&PhysicalTranslation, (With<Player>, Without<Enemy>)>,
) -> Result {
    let player_pos = player_q.single()?.truncate();

    let enemy_positions = enemy_q
        .iter()
        .map(|t| t.1.truncate())
        .collect::<Vec<Vec2>>();

    for (mut controller, physics_translation, root, halt, charge, jump) in enemy_q {
        if root.is_some() || halt.is_some() || charge.is_some() || jump.is_some() {
            //skip movement if enemy gets knockedback or is rooted
            controller.velocity = Vec3::ZERO;
        } else {
            let enemy_pos = physics_translation.truncate();

            let to_player = player_pos - enemy_pos;
            if to_player.length_squared() <= 0.0001 {
                controller.velocity = Vec3::ZERO;
                continue;
            }
            let direction = to_player.normalize();

            let separation_force = separation_force_calc(&enemy_positions, enemy_pos, player_pos);

            let movement = (direction + separation_force).normalize();
            controller.velocity = movement.extend(0.0);
        }
    }

    Ok(())
}

//Calc is short for calculator btw
fn separation_force_calc(enemy_positions: &Vec<Vec2>, own_pos: Vec2, player_pos: Vec2) -> Vec2 {
    let mut separation_force = Vec2::ZERO;
    for &other_pos in enemy_positions {
        // skip ourselves
        if other_pos == own_pos {
            continue;
        }
        // Check if the distance between enemy `A` and all other enemies is less than the
        // `SEPARATION_RADIUS`. If so, push enemy `A` away from the other enemy to maintain spacing.
        let distance = own_pos.distance(other_pos);
        if distance < SEPARATION_RADIUS {
            let push_dir = (own_pos - other_pos).normalize();
            let push_strength = (SEPARATION_RADIUS - distance) / SEPARATION_RADIUS;
            separation_force += push_dir * push_strength * SEPARATION_FORCE;
        }
    }
    // Separation force calculation for the player
    let distance_to_player = own_pos.distance(player_pos);
    if distance_to_player < SEPARATION_RADIUS {
        let push_dir = (own_pos - player_pos).normalize();
        let push_strength = (SEPARATION_RADIUS - distance_to_player) / SEPARATION_RADIUS;
        separation_force += push_dir * push_strength * SEPARATION_FORCE;
    }

    separation_force
}

fn enemy_colliding_detection(
    mut enemy_query: Query<
        (
            &PhysicalTranslation,
            Entity,
            Option<&mut Charge>,
            Option<&Jump>,
        ),
        (With<Enemy>, Without<Colliding>),
    >,
    player_query: Query<&PhysicalTranslation, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (enemy_pos, enemy, charge, jump) in &mut enemy_query {
        let distance_to_player = enemy_pos.distance(player_pos.0);

        if distance_to_player <= SEPARATION_RADIUS {
            //Charging enemies handle collision themself
            if let Some(mut charge) = charge {
                if charge.active && !charge.hit_target {
                    charge.hit_target = true;
                    commands.trigger(SprinterAbilityHitEvent(enemy));
                }
            } else if jump.is_some() {
                //Jumping enemies can't collide with player
                continue;
            } else {
                commands.entity(enemy).insert(Colliding);
            }
        }
    }
    Ok(())
}

fn enemy_stop_colliding_detection(
    enemy_query: Query<(&PhysicalTranslation, Entity), (With<Enemy>, With<Colliding>)>,
    player_query: Query<&PhysicalTranslation, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy) in &enemy_query {
        let distance_to_player = enemy_pos.distance(player_pos.0);

        if distance_to_player > SEPARATION_RADIUS {
            commands.entity(enemy).remove::<Colliding>();
        }
    }
    Ok(())
}

fn enemy_push_detection(
    enemy_query: Query<(&PhysicalTranslation, Entity, Option<&Charge>, Option<&Jump>), With<Enemy>>,
    player_query: Query<&PhysicalTranslation, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy, charge, jump) in &enemy_query {
        //Player cant push charging or jumping enemies
        if charge.is_some() || jump.is_some() {
            continue;
        }
        let distance_to_player = enemy_pos.distance(player_pos.0);

        if distance_to_player <= SEPARATION_RADIUS - 5.0 {
            commands.trigger(PlayerPushingEvent(enemy));
        }
    }
    Ok(())
}

fn enemy_pushing(
    trigger: On<PlayerPushingEvent>,
    player_query: Query<
        (&PhysicalTranslation, &MovementController),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_query: Query<(&PhysicalTranslation, &mut MovementController, Entity), With<Enemy>>,
) -> Result {
    let push_entity = trigger.event().0;
    let (player_pos, player_mc) = player_query.single()?;

    for (enemy_pos, mut forces, enemy_entity) in &mut enemy_query {
        if enemy_entity == push_entity {
            let dir = (enemy_pos.0 - player_pos.0).normalize();
            forces.apply_knockback_from_source(dir * PUSH_FORCE, player_mc);
        }
    }

    Ok(())
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
    mut enemy_q: Query<(&mut Health, &Transform), (With<Enemy>, Without<Despawn>)>,
    mut commands: Commands,
) {
    let enemy_entity = trigger.entity_hit;

    if let Ok((mut health, transform)) = enemy_q.get_mut(enemy_entity) {
        health.0 -= trigger.dmg;
        if health.0 <= 0.0 {
            commands.trigger(EnemyDeathEvent(*transform));
            commands.entity(enemy_entity).insert(Despawn);
        }
    }
}

fn enemy_get_pushed_from_hit(
    trigger: On<EnemyKnockbackEvent>,
    mut enemy_q: Query<
        (&mut MovementController, Option<&Charge>),
        (With<Enemy>, Without<PlayerProjectile>),
    >,
    projectile_q: Query<&MovementController, (With<PlayerProjectile>, Without<Enemy>)>,
) -> Result {
    let enemy_entity = trigger.entity_hit;
    let projectile_entity = trigger.spell_entity;
    let projectile_mc = projectile_q.get(projectile_entity)?;

    let proj_world_vel = projectile_mc.velocity * projectile_mc.speed;
    if proj_world_vel.length_squared() <= 1e-6 {
        return Ok(());
    }

    let dir = proj_world_vel.normalize();

    if let Ok((mut enemy_move, charge)) = enemy_q.get_mut(enemy_entity) {
        if charge.is_some() {
            // Charging enemies cannot be knocked back
            return Ok(());
        }

        enemy_move.apply_knockback_from_source(dir, projectile_mc);
    }

    Ok(())
}

fn enemy_despawner(enemy_q: Query<Entity, (With<Enemy>, With<Despawn>)>, mut commands: Commands) {
    for enemy in &enemy_q {
        commands.entity(enemy).despawn();
    }
}

//Inserts Halt if enemy is in range to player and is ranged
fn enemy_range_keeper(
    enemy_q: Query<
        (Entity, &PhysicalTranslation, &Range, Option<&Halt>),
        (With<Enemy>, With<Ranged>),
    >,
    player_q: Query<&PhysicalTranslation, With<Player>>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_q.single()?.truncate();

    for (enemy, physics_translation, range, halt) in &enemy_q {
        let enemy_pos = physics_translation.truncate();
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
            &PhysicalTranslation,
            Option<&Halt>,
            Option<&Range>,
        ),
        With<Enemy>,
    >,
    player_q: Query<&PhysicalTranslation, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
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
                    let distance = player_q.single()?.truncate().distance(transform.truncate());
                    if let Some(range) = range
                        && range.0 >= distance
                    {
                        commands.trigger(SprinterAttackEvent(enemy));
                    }
                }
                //We calculate only in the case so we dont cluter the update loop with unneeded calculations
                EnemyType::Jumper => {
                    let distance = player_q.single()?.truncate().distance(transform.truncate());
                    info!(distance);
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
    player_q: Query<&PhysicalTranslation, With<Player>>,
    projectiles: Query<&EnemyProjectiles>,
    projectile_q: Query<&PhysicalTranslation, With<EnemyProjectile>>,
    mut commands: Commands,
) -> Result {
    let player_transform = player_q.single()?;
    let player_pos = player_transform.truncate();
    // Get all enemies
    for (enemy, enemy_type) in &enemy_q {
        // Get each projectile of this enemy
        for projectile in projectiles.iter_descendants(enemy) {
            // Get position of this particular projectile
            let projectile_pos = projectile_q.get(projectile)?.truncate();

            //Check if player is hit by this projectile
            if (player_pos.distance(projectile_pos) - (SPELL_SIZE / 2.0)) <= (PLAYER_SIZE / 2.0) {
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
    let player_pos = player_q.single()?.translation.truncate();

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
