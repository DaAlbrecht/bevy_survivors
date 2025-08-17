use bevy::prelude::*;

use crate::gameplay::{
    attacks::{Cooldown, ExplosionRadius, ProjectileConfig, Spell, SpellType},
    enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
    player::{Player, spawn_player},
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Fireball,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    ProjectileConfig{
        speed: 600.,
        knockback: 1500.,
        damage: 5.,
        projectile_count: 1.,
    },
    ExplosionRadius(100.),
    Name::new("Fireball")
)]
pub(crate) struct Fireball;

#[derive(Event)]
pub(crate) struct FireballAttackEvent;

#[derive(Event)]
pub(crate) struct FireballHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, (spawn_fireball).after(spawn_player));
    app.add_observer(spawn_fireball_projectile);
    app.add_observer(fireball_hit);
}

fn spawn_fireball(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    let fireball = commands.spawn((Fireball,)).id();

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
            config.add_projectile(
                direction.extend(0.),
                player_pos.translation,
                SpellType::Fireball,
            ),
        ));
    }

    Ok(())
}

fn fireball_hit(
    trigger: Trigger<FireballHitEvent>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    mut commands: Commands,
    fireball_q: Query<&ExplosionRadius, With<Fireball>>,
) -> Result {
    let enemy_entity = trigger.enemy;
    let spell_entity = trigger.projectile;
    let explosion_radius = fireball_q.single()?;

    //Deal damage
    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy_entity,
        spell_entity,
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

            if distance < explosion_radius.0 {
                commands.trigger(EnemyDamageEvent {
                    entity_hit: other_enemy,
                    spell_entity,
                });
            }
        }
    }

    //Knockback
    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy_entity,
        spell_entity,
    });

    commands.entity(spell_entity).despawn();

    Ok(())
}
