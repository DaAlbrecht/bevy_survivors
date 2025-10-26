use bevy_enhanced_input::action::Action;

use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

use crate::{
    PLAYER_SIZE, SPELL_SIZE,
    gameplay::{
        Health,
        enemy::{
            jumper::JumperAttackEvent,
            shooter::{ShooterAttackEvent, ShooterProjectileHitEvent},
            sprinter::{SprinterAbilityHitEvent, SprinterAttackEvent},
        },
        player::{Direction, Move, PlayerHitEvent},
        spells::{
            CastSpell, Cooldown, Damage, Despawn, Halt, Range, Root, Spell, SpellDuration,
            SpellTick,
        },
    },
    screens::Screen,
};

use super::player::Player;

use super::spells::{Knockback, PlayerProjectile};

mod jumper;
mod shooter;
mod sprinter;
mod walker;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        jumper::plugin,
        walker::plugin,
        shooter::plugin,
        sprinter::plugin,
    ));

    app.add_systems(
        Update,
        (
            enemy_colliding_detection,
            enemy_stop_colliding_detection,
            enemy_push_detection,
            move_enemy_from_knockback,
            attack,
            enemy_despawner,
            enemy_timer_handle,
            move_enemy_projectile,
            projectile_hit_detection,
            enemy_movement,
            enemy_range_keeper,
            terrain_manager,
        )
            .run_if(in_state(Screen::Gameplay)),
    )
    .add_observer(enemy_pushing)
    .add_observer(enemy_take_dmg)
    .add_observer(enemy_get_pushed_from_hit);
}

const SPAWN_RADIUS: f32 = 200.0;
const SEPARATION_RADIUS: f32 = 40.;
const SEPARATION_FORCE: f32 = 10.;
const ENEMY_DMG_STAT: f32 = 5.;
const RANGE_BUFFER: f32 = 50.0;

#[derive(Component, Reflect)]
pub(crate) struct Speed(pub f32);

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

//type shenanigans
#[derive(Component)]
pub(crate) struct KnockbackDirection(pub Direction);

#[derive(Component)]
pub(crate) struct ProjectileSpeed(pub f32);

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

#[derive(Component)]
pub(crate) enum EnemyType {
    Shooter,
    Sprinter,
    Jumper,
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

fn enemy_colliding_detection(
    mut enemy_query: Query<
        (&Transform, Entity, Option<&mut Charge>, Option<&Jump>),
        (With<Enemy>, Without<Colliding>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (enemy_pos, enemy, charge, jump) in &mut enemy_query {
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

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
    enemy_query: Query<(&mut Transform, Entity), (With<Enemy>, With<Colliding>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy) in &enemy_query {
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

        if distance_to_player > SEPARATION_RADIUS {
            commands.entity(enemy).remove::<Colliding>();
        }
    }
    Ok(())
}

fn enemy_push_detection(
    enemy_query: Query<(&mut Transform, Entity, Option<&Charge>, Option<&Jump>), With<Enemy>>,
    player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_query.single()?;

    for (&enemy_pos, enemy, charge, jump) in &enemy_query {
        //Player cant push charging or jumping enemies
        if charge.is_some() || jump.is_some() {
            continue;
        }
        let distance_to_player = enemy_pos.translation.distance(player_pos.translation);

        if distance_to_player <= SEPARATION_RADIUS - 5.0 {
            commands.trigger(PlayerPushingEvent(enemy));
        }
    }
    Ok(())
}

fn enemy_pushing(
    trigger: On<PlayerPushingEvent>,
    move_action: Single<&Action<Move>>,
    mut enemy_query: Query<(&mut Transform, Entity), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
) {
    let push_entity = trigger.event().0;

    for (mut enemy_pos, enemy_entity) in &mut enemy_query {
        if enemy_entity == push_entity {
            enemy_pos.translation += move_action.extend(0.0) * time.delta_secs();
        }
    }
}

//maybe refactor with timer handle later?
//this is where the player get damaged form touching an enemy
fn attack(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_dmg_timer_q: Query<&mut DamageCooldown, (With<Enemy>, With<Colliding>)>,
) {
    for mut timer in &mut enemy_dmg_timer_q {
        if timer.0.tick(time.delta()).just_finished() {
            commands.trigger(PlayerHitEvent {
                dmg: ENEMY_DMG_STAT,
            });
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
    mut enemy_q: Query<(&mut Knockback, &mut KnockbackDirection, Option<&Charge>), With<Enemy>>,
    knockback: Query<&Knockback, (With<Spell>, Without<Enemy>)>,
    projectile_direction: Query<&Direction, With<PlayerProjectile>>,
    spells: Query<&CastSpell>,
) -> Result {
    let enemy_entity = trigger.entity_hit;
    let projectile_entity = trigger.spell_entity;

    //Get the Spell of this projectile, each projectile, this is a 1-many relationship
    let spell = spells
        .iter_ancestors(projectile_entity)
        .next()
        .expect("there should always only be one ancestor spell for each projectile");

    let knockback = knockback.get(spell)?.0;

    let direction = projectile_direction.get(projectile_entity)?.0;

    if let Ok((mut enemy_knockback, mut enemy_knockback_direction, charge)) =
        enemy_q.get_mut(enemy_entity)
    {
        //Charging enemies cant be knockedback
        if charge.is_some() {
            return Ok(());
        }

        enemy_knockback.0 = knockback;
        //type shenanigans continue
        enemy_knockback_direction.0.0 = direction;
    }

    Ok(())
}

fn move_enemy_from_knockback(
    mut enemy_q: Query<(&mut Knockback, &mut Transform, &KnockbackDirection), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut enemy_knockback, mut enemy_transform, enemy_direction) in &mut enemy_q {
        if enemy_knockback.0 > 0.0 {
            //Very sorry for the type shenanigans at this point tbh
            enemy_transform.translation +=
                enemy_knockback.0 * enemy_direction.0.0 * time.delta_secs();

            //reduce knockback speed each frame (friction)
            enemy_knockback.0 *= 0.95;

            //Stop if slow
            if enemy_knockback.0 <= 1.0 {
                enemy_knockback.0 = 0.0;
            }
        }
    }
}

fn enemy_despawner(enemy_q: Query<Entity, (With<Enemy>, With<Despawn>)>, mut commands: Commands) {
    for enemy in &enemy_q {
        commands.entity(enemy).despawn();
    }
}

//BUG: Enemies with Halt get dragged by the player refactor collison handle and halt
fn enemy_movement(
    enemy_q: Query<
        (
            &mut Transform,
            &Speed,
            &Knockback,
            Option<&Root>,
            Option<&Halt>,
            Option<&Charge>,
            Option<&Jump>,
        ),
        With<Enemy>,
    >,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) -> Result {
    let player_pos = player_q.single()?.translation.truncate();

    let enemy_positions = enemy_q
        .iter()
        .map(|t| t.0.translation.truncate())
        .collect::<Vec<Vec2>>();

    for (mut transform, speed, knockback, root, halt, charge, jump) in enemy_q {
        let enemy_pos = transform.translation.truncate();
        if knockback.0 > 1.0
            || root.is_some()
            || halt.is_some()
            || charge.is_some()
            || jump.is_some()
        {
            //skip movement if enemy gets knockedback or is rooted
            continue;
        }

        let direction = (player_pos - enemy_pos).normalize();

        let separation_force = separation_force_calc(&enemy_positions, enemy_pos, player_pos);

        let movement = (direction + separation_force).normalize() * (speed.0 * time.delta_secs());
        transform.translation += movement.extend(0.0);
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

//Inserts Halt if enemy is in range to player and is ranged
fn enemy_range_keeper(
    enemy_q: Query<(Entity, &Transform, &Range, Option<&Halt>), (With<Enemy>, With<Ranged>)>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let player_pos = player_q.single()?.translation.truncate();

    for (enemy, transform, range, halt) in &enemy_q {
        let enemy_pos = transform.translation.truncate();
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
    mut cooldown_q: Query<(Entity, &mut Cooldown, &EnemyType, Option<&Halt>), With<Enemy>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (enemy, mut cooldown_timer, enemy_tye, halt) in &mut cooldown_q {
        cooldown_timer.0.tick(time.delta());

        if cooldown_timer.0.is_finished() {
            match enemy_tye {
                EnemyType::Shooter => {
                    if halt.is_some() {
                        commands.trigger(ShooterAttackEvent(enemy));
                    }
                }
                EnemyType::Sprinter => commands.trigger(SprinterAttackEvent(enemy)),
                EnemyType::Jumper => commands.trigger(JumperAttackEvent(enemy)),
            }

            cooldown_timer.0.reset();
        }
    }
}

fn move_enemy_projectile(
    enemy_q: Query<(Entity, &ProjectileSpeed), With<Enemy>>,
    projectiles: Query<&EnemyProjectiles>,
    mut projectile_q: Query<(&mut Transform, &Direction), (With<EnemyProjectile>, Without<Halt>)>,
    time: Res<Time>,
) {
    //Loop over all types of enemies
    for (enemy, speed) in &enemy_q {
        // Iter over each projectile for this given enemy type
        for projectile in projectiles.iter_descendants(enemy) {
            let Ok((mut transform, direction)) = projectile_q.get_mut(projectile) else {
                continue;
            };

            let movement = direction.0 * speed.0 * time.delta_secs();
            transform.translation += movement;
        }
    }
}

fn projectile_hit_detection(
    enemy_q: Query<(Entity, &EnemyType), With<Enemy>>,
    player_q: Query<&Transform, With<Player>>,
    projectiles: Query<&EnemyProjectiles>,
    projectile_q: Query<&Transform, With<EnemyProjectile>>,
    mut commands: Commands,
) -> Result {
    let player_transform = player_q.single()?;
    let player_pos = player_transform.translation.truncate();
    // Get all enemies
    for (enemy, enemy_type) in &enemy_q {
        // Get each projectile of this enemy
        for projectile in projectiles.iter_descendants(enemy) {
            // Get position of this particular projectile
            let projectile_pos = projectile_q.get(projectile)?.translation.truncate();

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
