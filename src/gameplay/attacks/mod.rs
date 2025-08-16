use std::time::Duration;

use bevy::prelude::*;

use crate::{
    ENEMY_SIZE, SPELL_SIZE,
    gameplay::{
        attacks::{
            fireball::{FireballAttackEvent, FireballHitEvent, FireballPlugin},
            lightning::{LightningAttackEvent, LightningPlugin},
            orbs::{OrbAttackEvent, OrbPlugin},
            scale::{ScaleAttackEvent, ScaleHitEvent, ScalePlugin},
        },
        enemy::{Enemy, Speed},
        player::{Direction, Player},
    },
};

pub mod fireball;
pub mod lightning;
pub mod orbs;
pub mod scale;

pub(crate) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ScalePlugin, FireballPlugin, LightningPlugin, OrbPlugin));

        app.add_systems(Update, (update_attack_timers, projectile_hit_detection));
        app.add_systems(FixedUpdate, move_player_projectile);
    }
}

#[derive(Component)]
pub(crate) struct PlayerProjectile;

#[derive(Component, Default)]
pub(crate) struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Knockback(pub f32);

#[derive(Component)]
pub(crate) struct Damage(pub f32);

#[derive(Component)]
pub(crate) struct Range(pub f32);

#[derive(Component)]
pub(crate) struct Attack;

#[derive(Component)]
pub(crate) struct SpellDuration(pub Timer);

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) enum SpellType {
    Scale,
    Fireball,
    Lightning,
    Orb,
}

#[derive(Component)]
pub(crate) struct ProjectileConfig {
    speed: f32,
    knockback: f32,
    damage: f32,
}

fn move_player_projectile(
    mut bullet_pos_q: Query<
        (&mut Transform, &Speed, &Direction),
        (With<PlayerProjectile>, Without<Player>),
    >,
    time: Res<Time>,
) {
    for (mut bullet_pos, bullet_speed, bullet_direction) in &mut bullet_pos_q {
        let movement = bullet_direction.0 * bullet_speed.0 * time.delta_secs();
        bullet_pos.translation += movement;
    }
}

fn projectile_hit_detection(
    enemy_query: Query<(&Transform, Entity), (With<Enemy>, Without<PlayerProjectile>)>,
    projectile_query: Query<(&Transform, Entity, &SpellType), With<PlayerProjectile>>,
    mut commands: Commands,
) {
    for (&projectile_pos, projectile_entity, &spell_type) in &projectile_query {
        for (&enemy_pos, enemy_entity) in &enemy_query {
            if (projectile_pos.translation.distance(enemy_pos.translation) - (SPELL_SIZE / 2.0))
                <= ENEMY_SIZE / 2.0
            {
                trigger_hit_event(&mut commands, spell_type, projectile_entity, enemy_entity);
            }
        }
    }
}

fn update_attack_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut Cooldown, With<Attack>>,
    mut durations: Query<&mut SpellDuration, With<Attack>>,
) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }
}

pub(crate) fn trigger_attack_event(commands: &mut Commands, spell_type: SpellType) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleAttackEvent),
        SpellType::Fireball => commands.trigger(FireballAttackEvent),
        SpellType::Lightning => commands.trigger(LightningAttackEvent),
        SpellType::Orb => commands.trigger(OrbAttackEvent),
    }
}

pub(crate) fn trigger_hit_event(
    commands: &mut Commands,
    spell_type: SpellType,
    projectile: Entity,
    enemy: Entity,
) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleHitEvent { enemy, projectile }),
        SpellType::Fireball => commands.trigger(FireballHitEvent { enemy, projectile }),
        SpellType::Orb => {}
        _ => {}
    }
}
