use bevy::prelude::*;

use crate::gameplay::{
    attacks::scale::ScaleAttackEvent,
    enemy::Speed,
    player::{Direction, Player},
};

pub mod scale;

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
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
pub struct Attack;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum SpellType {
    Scale,
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

pub fn trigger_attack_event(commands: &mut Commands, spell_type: SpellType) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleAttackEvent),
    }
}
