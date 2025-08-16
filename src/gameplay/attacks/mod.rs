use bevy::prelude::*;

use crate::{
    ENEMY_SIZE, SPELL_SIZE,
    gameplay::{
        attacks::{
            fireball::{FireballAttackEvent, FireballHitEvent},
            lightning::LightningAttackEvent,
            orbs::OrbAttackEvent,
            scale::{ScaleAttackEvent, ScaleHitEvent},
        },
        enemy::{Enemy, Speed},
        player::{Direction, Player},
    },
    screens::Screen,
};

pub mod fireball;
pub mod lightning;
pub mod orbs;
pub mod scale;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        scale::plugin,
        fireball::plugin,
        lightning::plugin,
        orbs::plugin,
    ));

    app.add_systems(
        Update,
        (attack, update_attack_timers, projectile_hit_detection).run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(FixedUpdate, move_projectile);
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

fn attack(mut attack_q: Query<(&mut Cooldown, &SpellType), With<Attack>>, mut commands: Commands) {
    for (mut cooldown, &spell_type) in &mut attack_q {
        if cooldown.0.finished() {
            match spell_type {
                SpellType::Scale => commands.trigger(ScaleAttackEvent),
                SpellType::Fireball => commands.trigger(FireballAttackEvent),
                SpellType::Lightning => commands.trigger(LightningAttackEvent),
                SpellType::Orb => commands.trigger(OrbAttackEvent),
            }
            cooldown.0.reset();
        }
    }
}

fn move_projectile(
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

pub(crate) fn trigger_hit_event(
    commands: &mut Commands,
    spell_type: SpellType,
    projectile: Entity,
    enemy: Entity,
) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleHitEvent { enemy, projectile }),
        SpellType::Fireball => commands.trigger(FireballHitEvent { enemy, projectile }),
        _ => {}
    }
}
