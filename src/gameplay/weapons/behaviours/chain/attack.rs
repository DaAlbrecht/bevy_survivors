use bevy::{platform::collections::HashSet, prelude::*};
use bevy_seedling::sample::SamplePlayer;

use crate::{
    audio::SfxPool,
    gameplay::{
        enemy::Enemy,
        player::Player,
        weapons::{
            prelude::*, spec::apply::sfx::WeaponAttackSfx, systems::cooldown::WeaponDuration,
        },
    },
};

pub fn on_chain_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<
        (
            &ProjectileCount,
            &WeaponRange,
            &super::ChainLifetime,
            &WeaponProjectileVisuals,
            Option<&WeaponAttackSfx>,
        ),
        With<super::ChainAttack>,
    >,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemy_q: Query<(&Transform, Entity), (With<Enemy>, Without<Player>)>,
    weapon_stats_q: Query<(&HitSpec, &BaseDamage)>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((chain_count, chain_range, bolt_lifetime, projectile_visuals, sfx)) =
        weapon_q.get(weapon)
    else {
        return Ok(());
    };

    if let Some(weapon_sfx) = sfx {
        commands.spawn((SamplePlayer::new(weapon_sfx.0.clone()), SfxPool));
    }

    let player_pos = player_q.single()?;

    let mut current_source_pos = player_pos;
    let mut current_source_entity: Option<Entity> = None;
    let mut visited: HashSet<Entity> = HashSet::new();

    for _ in 0..chain_count.0 {
        let mut max_distance = chain_range.0;
        let mut closest: Option<(Entity, &Transform)> = None;

        for (enemy_pos, enemy) in &enemy_q {
            if Some(enemy) == current_source_entity {
                continue;
            }

            if visited.contains(&enemy) {
                continue;
            }

            let distance = current_source_pos
                .translation
                .truncate()
                .distance(enemy_pos.translation.truncate());

            if distance < max_distance {
                max_distance = distance;
                closest = Some((enemy, enemy_pos));
            }
        }

        let Some((enemy, enemy_pos)) = closest else {
            break;
        };

        let direction = (enemy_pos.translation - current_source_pos.translation).truncate();
        let length = direction.length();
        let angle = direction.y.atan2(direction.x);
        let anchor_point = current_source_pos.translation.truncate() + direction * 0.5;

        let mut bolt = commands.spawn((
            Name::new("ChainProj"),
            Transform {
                translation: anchor_point.extend(10.0),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::new(length / 16.0, 1.0, 1.0),
            },
            WeaponDuration(Timer::from_seconds(bolt_lifetime.0, TimerMode::Once)),
        ));

        projectile_visuals.0.apply_ec(&mut bolt);

        let (hit, dmg) = weapon_stats_q.get(weapon)?;
        commands.trigger(WeaponHitEvent {
            entity: weapon,
            target: enemy,
            hit_pos: enemy_pos.translation,
            dmg: dmg.0,
            damage_type: hit.damage_type,
            aoe: None,
            effects: hit.effects.clone(),
        });

        visited.insert(enemy);
        current_source_pos = enemy_pos;
        current_source_entity = Some(enemy);
    }

    Ok(())
}
