use bevy::prelude::*;

use crate::{
    PausableSystems,
    gameplay::{
        items::stats::{DerivedStats, RecalculateStats, StatId},
        player::InInventoryOf,
        weapons::{components::Weapon, systems::attack::WeaponAttack},
    },
    screens::Screen,
};

#[derive(Component, Default, Reflect)]
pub struct WeaponCooldownTimer(pub Timer);

#[derive(Component, Reflect)]
pub struct WeaponDurationTimer(pub Timer);

#[derive(Component, Reflect)]
pub struct WeaponBaseCooldown(pub f32);

// TODO: ADD IF WE HAVE A STAT FOR IT
// #[derive(Component, Reflect)]
// pub struct BaseDuration(pub f32);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (handle_timers, tick_despawn_after)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
    app.add_observer(update_weapon_cooldowns_on_stat_change);
}

fn handle_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut weapon_q: Query<(Entity, &mut WeaponCooldownTimer), With<Weapon>>,
    mut durations: Query<&mut WeaponDurationTimer>,
) {
    for (entity, mut cooldown) in &mut weapon_q {
        if cooldown.0.just_finished() {
            commands.trigger(WeaponAttack { entity });
        }
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }
}

pub fn tick_despawn_after(q: Query<(Entity, &mut WeaponDurationTimer)>, mut commands: Commands) {
    for (e, t) in q {
        if t.0.is_finished() {
            commands.entity(e).despawn();
        }
    }
}

fn update_weapon_cooldowns_on_stat_change(
    trigger: On<RecalculateStats>,
    player_stats: Query<&DerivedStats>,
    weapon_q: Query<(&InInventoryOf, &WeaponBaseCooldown, &mut WeaponCooldownTimer), With<Weapon>>,
) {
    let Ok(stats) = player_stats.get(trigger.entity) else {
        return;
    };

    let attack_speed = stats.0.get(StatId::AttackSpeed);

    for (inventory_of, base_cooldown, mut cooldown) in weapon_q {
        if inventory_of.0 == trigger.entity {
            let new_duration = base_cooldown.0 / attack_speed;
            cooldown
                .0
                .set_duration(std::time::Duration::from_secs_f32(new_duration));
        }
    }
}
