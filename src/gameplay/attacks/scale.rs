use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::{
    attacks::{ProjectileConfig, Spell, SpellType},
    enemy::{EnemyDamageEvent, EnemyKnockbackEvent},
    player::{AddToInventory, Player, spawn_player},
};

use super::Cooldown;

use bevy_rand::{global::GlobalEntropy, prelude::WyRand};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Scale,
    Cooldown(Timer::from_seconds(1., TimerMode::Once)),
    ProjectileConfig{
        speed: 600.,
        knockback: 1500.,
        damage: 5.,
        projectile_count: 1.,
    },
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
    app.add_systems(Startup, (add_scale_spell).after(spawn_player));

    app.add_observer(spawn_scale_projectile);
    app.add_observer(scale_hit);
}

fn add_scale_spell(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    commands.spawn((Scale, AddToInventory(player)));

    Ok(())
}

fn spawn_scale_projectile(
    _trigger: Trigger<ScaleAttackEvent>,
    player_pos_q: Query<&Transform, With<Player>>,
    config_q: Query<&ProjectileConfig, With<Scale>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_pos_q.single()?;
    let config = config_q.single()?;
    let random_angle: f32 = rng.gen_range(0.0..(2. * PI));
    let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

    commands.spawn((
        Sprite {
            image: asset_server.load("Bullet.png"),
            ..default()
        },
        config.add_projectile(direction, player_pos.translation, SpellType::Scale),
    ));

    Ok(())
}

fn scale_hit(trigger: Trigger<ScaleHitEvent>, mut commands: Commands) {
    let enemy = trigger.enemy;
    let spell_entity = trigger.projectile;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy,
        spell_entity,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy,
        spell_entity,
    });

    commands.entity(spell_entity).despawn();
}
