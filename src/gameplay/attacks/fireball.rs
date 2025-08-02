use bevy::prelude::*;

use crate::gameplay::{
    attacks::{Attack, Cooldown, Damage, Knockback, PlayerProjectile, ProjectileConfig, SpellType},
    enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent, Speed},
    player::{Direction, Player, spawn_player},
};

const FIREBALL_BASE_COOLDOWN: f32 = 5.0;
const FIREBALL_BASE_SPEED: f32 = 600.0;
const FIREBALL_BASE_KNOCKBACK: f32 = 1500.0;
const FIREBALL_BASE_DMG: f32 = 5.0;
const EXPLOSION_RADIUS: f32 = 50.0;

#[derive(Component)]
pub struct Fireball;

#[derive(Event)]
pub struct FireballAttackEvent;

#[derive(Event)]
pub struct FireballHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_fireball).after(spawn_player));
        app.add_observer(spawn_fireball_projectile);
        app.add_observer(fireball_hit);
    }
}

fn spawn_fireball(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    let fireball = commands
        .spawn((
            Attack,
            Fireball,
            SpellType::Fireball,
            Cooldown(Timer::from_seconds(FIREBALL_BASE_COOLDOWN, TimerMode::Once)),
            ProjectileConfig {
                speed: FIREBALL_BASE_SPEED,
                knockback: FIREBALL_BASE_KNOCKBACK,
                damage: FIREBALL_BASE_DMG,
            },
            Name::new("Fireball"),
        ))
        .id();

    commands.entity(player).add_child(fireball);

    Ok(())
}

fn spawn_fireball_projectile(
    _trigger: Trigger<FireballAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    config_q: Query<&ProjectileConfig, With<Fireball>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player_q.single()?;
    let config = config_q.single()?;

    let mut min_distance = f32::MAX;
    let mut closest_enemy: Option<&Transform> = None;

    for enemy_pos in &enemy_q {
        let distance = player_pos
            .translation
            .truncate()
            .distance(enemy_pos.translation.truncate());

        if distance < min_distance {
            min_distance = distance;
            closest_enemy = Some(enemy_pos);
        }
    }

    if let Some(enemy_pos) = closest_enemy {
        let direction = (enemy_pos.translation - player_pos.translation)
            .truncate()
            .normalize();

        commands.spawn((
            Sprite {
                image: asset_server.load("Fireball.png"),
                ..default()
            },
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
            PlayerProjectile,
            SpellType::Fireball,
            Speed(config.speed),
            Knockback(config.knockback),
            Damage(config.damage),
            Direction(direction.extend(0.0)),
            Name::new("FireballProjectile"),
        ));
    }

    Ok(())
}

fn fireball_hit(
    trigger: Trigger<FireballHitEvent>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    mut commands: Commands,
) {
    let enemy_entity = trigger.enemy;
    let projectile_entity = trigger.projectile;

    //Deal damage and knockback
    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy_entity,
        spell_entity: projectile_entity,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy_entity,
        spell_entity: projectile_entity,
    });

    //Deal damage to all enemys in explosion radius
    if let Ok((enemy_pos, _)) = enemy_q.get(enemy_entity) {
        for (other_enemy_pos, other_enemy) in &enemy_q {
            if other_enemy_pos == enemy_pos {
                //Skipp enemy hit
                continue;
            }
            let distance = enemy_pos
                .translation
                .truncate()
                .distance(other_enemy_pos.translation.truncate());

            if distance < EXPLOSION_RADIUS {
                commands.trigger(EnemyDamageEvent {
                    entity_hit: other_enemy,
                    spell_entity: projectile_entity,
                });
            }
        }
    }

    commands.entity(projectile_entity).despawn();
}
