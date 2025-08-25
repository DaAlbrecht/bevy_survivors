use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::{
    enemy::{EnemyDamageEvent, EnemyKnockbackEvent, Speed},
    player::{Direction, Player},
    spells::{CastSpell, Damage, Knockback, PlayerProjectile, Spell, SpellType},
};

use super::Cooldown;

use bevy_rand::{global::GlobalEntropy, prelude::WyRand};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Scale,
    Cooldown(Timer::from_seconds(1., TimerMode::Once)),
    Speed(600.),
    Knockback(1500.),
    Damage(5.),
    Name::new("Scale")
)]
pub(crate) struct Scale;

#[derive(Event)]
pub(crate) struct ScaleAttackEvent;

#[derive(Event)]
pub(crate) struct ScaleHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(spawn_scale_projectile);
    app.add_observer(scale_hit);
}

fn spawn_scale_projectile(
    _trigger: Trigger<ScaleAttackEvent>,
    player_pos_q: Query<&Transform, With<Player>>,
    scale: Query<Entity, With<Scale>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_pos_q.single()?;
    let scale = scale.single()?;

    let random_angle: f32 = rng.gen_range(0.0..(2. * PI));
    let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

    commands.spawn((
        Name::new("scale projectile"),
        Sprite {
            image: asset_server.load("Bullet.png"),
            ..default()
        },
        CastSpell(scale),
        Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
        Direction(direction),
        PlayerProjectile,
    ));

    Ok(())
}

fn scale_hit(
    trigger: Trigger<ScaleHitEvent>,
    mut commands: Commands,
    scale_dmg: Query<&Damage, With<Scale>>,
) -> Result {
    let enemy = trigger.enemy;
    let spell_entity = trigger.projectile;
    let dmg = scale_dmg.single()?.0;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy,
        dmg,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy,
        spell_entity,
    });

    commands.entity(spell_entity).despawn();
    Ok(())
}
