use bevy::prelude::*;
use rand::Rng;

use crate::gameplay::{
    Despawn, Health,
    damage_numbers::{DamageMessage, DamageType},
    enemy::Enemy,
    items::stats::{DerivedStats, StatId, Stats},
    simple_animation::HurtAnimationTimer,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(enemy_take_dmg);
}

#[derive(Event, Reflect)]
pub(crate) struct EnemyDamageEvent {
    pub entity_hit: Entity,
    pub dmg: f32,
    pub damage_type: DamageType,
}

#[derive(Event, Reflect)]
pub(crate) struct EnemyKnockbackEvent {
    pub entity_hit: Entity,
    pub strength: f32,
    pub dir: Vec2,
}

#[derive(Event, Reflect)]
pub(crate) struct EnemyDeathEvent(pub Transform);

fn enemy_take_dmg(
    trigger: On<EnemyDamageEvent>,
    stats_q: Single<&DerivedStats>,
    mut damage_writer: MessageWriter<DamageMessage>,
    mut enemy_q: Query<(&mut Health, &Transform), (With<Enemy>, Without<Despawn>)>,
    mut commands: Commands,
) {
    let enemy_entity = trigger.entity_hit;
    let stats = stats_q.0;

    let (dmg, crit) = damage_calculator(trigger.dmg, &stats);

    commands
        .entity(enemy_entity)
        .insert(HurtAnimationTimer::default());

    if let Ok((mut health, transform)) = enemy_q.get_mut(enemy_entity) {
        health.0 -= dmg;

        //TODO: DamageType only really used for effects
        damage_writer.write(DamageMessage {
            amount: dmg as i32,
            world_pos: transform.translation.truncate(),
            crit,
            damage_type: trigger.damage_type,
        });

        if health.0 <= 0.0 {
            commands.trigger(EnemyDeathEvent(*transform));
            commands.entity(enemy_entity).insert(Despawn);
        }
    }
}

pub fn damage_calculator(base_damage: f32, stats: &Stats) -> (f32, bool) {
    let is_crit = {
        let mut rng = rand::rng();
        rng.random_bool(stats.get(StatId::CritChance).into())
    };

    let crit_multiplier = if is_crit {
        stats.get(StatId::CritDamage)
    } else {
        1.0
    };

    let dmg = stats.get(StatId::Attack) + base_damage * crit_multiplier;

    (dmg, is_crit)
}
