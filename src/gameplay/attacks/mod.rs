use bevy::prelude::*;

use crate::gameplay::{
    attacks::{fireball::FireballAttackEvent, scale::ScaleAttackEvent},
    enemy::Speed,
    player::{Direction, Player},
};

pub mod fireball;
pub mod scale;

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_attack_cooldown);
        app.add_systems(FixedUpdate, move_player_spell);
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
}

#[derive(Component)]
pub struct ProjectileConfig {
    speed: f32,
    knockback: f32,
    damage: f32,
}

fn move_player_spell(
    mut bullet_pos_q: Query<
        (&mut Transform, &Speed, &Direction),
        (With<PlayerProjectile>, Without<Player>),
    >,
    time: Res<Time>,
) -> Result {
    for (mut bullet_pos, bullet_speed, bullet_direction) in &mut bullet_pos_q {
        let movement = bullet_direction.0 * bullet_speed.0 * time.delta_secs();
        bullet_pos.translation += movement;
    }

    Ok(())
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
    }
}
