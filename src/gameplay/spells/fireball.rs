use bevy::prelude::*;

use crate::gameplay::{
    Speed,
    enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
    player::{Direction, Player},
    spells::{
        CastSpell, Cooldown, Damage, ExplosionRadius, Knockback, PlayerProjectile, Spell, SpellType,
    },
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Fireball,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    Speed(600.),
    Knockback(1500.),
    Damage(5.),
    ExplosionRadius(100.),
    Name::new("Fireball")
)]
#[derive(Reflect)]
pub(crate) struct Fireball;

#[derive(Event, Reflect)]
pub(crate) struct FireballAttackEvent;

#[derive(Event, Reflect)]
pub(crate) struct FireballHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(spawn_fireball_projectile);
    app.add_observer(fireball_hit);
}

fn spawn_fireball_projectile(
    _trigger: On<FireballAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    fireball: Query<Entity, With<Fireball>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player_q.single()?;
    let fireball = fireball.single()?;

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
            Name::new("fireball projectile"),
            Sprite {
                image: asset_server.load("Fireball.png"),
                ..default()
            },
            CastSpell(fireball),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
            Direction(direction.extend(0.)),
            PlayerProjectile,
        ));
    }

    Ok(())
}

fn fireball_hit(
    trigger: On<FireballHitEvent>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    mut commands: Commands,
    explosion_radius: Query<(&ExplosionRadius, &Damage), With<Fireball>>,
) -> Result {
    let enemy_entity = trigger.enemy;
    let spell_entity = trigger.projectile;
    let (explosion_radius, dmg) = explosion_radius.single()?;

    let dmg = dmg.0;

    //Deal damage
    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy_entity,
        dmg,
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
                    dmg,
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
