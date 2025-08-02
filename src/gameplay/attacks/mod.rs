use bevy::prelude::*;

use crate::{
    ENEMY_SIZE, SPELL_SIZE,
    gameplay::{
        attacks::{
            fireball::{FireballAttackEvent, FireballHitEvent},
            lightning::LightningAttackEvent,
            scale::{ScaleAttackEvent, ScaleHitEvent},
        },
        enemy::{Enemy, Speed},
        player::{Direction, Player},
    },
};

pub mod fireball;
pub mod lightning;
pub mod scale;

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_attack_cooldown, projectile_hit_detection));
        app.add_systems(FixedUpdate, move_player_projectile);
    }
}

#[derive(Component)]
pub struct PlayerProjectile;

#[derive(Component, Default)]
pub struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub struct Knockback(pub f32);

#[derive(Component)]
pub struct Damage(pub f32);

#[derive(Component)]
pub struct Attack;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum SpellType {
    Scale,
    Fireball,
    Lightning,
}

#[derive(Component)]
pub struct ProjectileConfig {
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

fn update_attack_cooldown(time: Res<Time>, mut cooldowns: Query<&mut Cooldown, With<Attack>>) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }
}

pub fn trigger_attack_event(commands: &mut Commands, spell_type: SpellType) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleAttackEvent),
        SpellType::Fireball => commands.trigger(FireballAttackEvent),
        SpellType::Lightning => commands.trigger(LightningAttackEvent),
    }
}

pub fn trigger_hit_event(
    commands: &mut Commands,
    spell_type: SpellType,
    projectile: Entity,
    enemy: Entity,
) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleHitEvent { enemy, projectile }),
        SpellType::Fireball => commands.trigger(FireballHitEvent { enemy, projectile }),
        SpellType::Lightning => {}
    }
}
