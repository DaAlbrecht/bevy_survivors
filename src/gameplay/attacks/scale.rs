use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::{
    attacks::{Attack, Damage, ProjectileConfig, SpellType},
    enemy::{EnemyDamageEvent, EnemyKnockbackEvent, Speed},
    player::{Direction, Player, spawn_player},
};

use super::{Cooldown, Knockback, PlayerProjectile};

use bevy_rand::{global::GlobalEntropy, prelude::WyRand};

const SCALE_BASE_COOLDOWN: f32 = 1.0;
const SCALE_BASE_SPEED: f32 = 600.0;
const SCALE_BASE_KNOCKBACK: f32 = 1500.0;
const SCALE_BASE_DMG: f32 = 5.0;

#[derive(Component)]
pub(crate) struct Scale;

#[derive(Event)]
pub(crate) struct ScaleAttackEvent;

#[derive(Event)]
pub(crate) struct ScaleHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) struct ScalePlugin;

impl Plugin for ScalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_scale).after(spawn_player));

        app.add_observer(spawn_scale_projectile);
        app.add_observer(scale_hit);
    }
}

fn spawn_scale(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    let scale = commands
        .spawn((
            Attack,
            Scale,
            SpellType::Scale,
            Cooldown(Timer::from_seconds(SCALE_BASE_COOLDOWN, TimerMode::Once)),
            //Lets us change all projectile stats at one place
            ProjectileConfig {
                speed: SCALE_BASE_SPEED,
                knockback: SCALE_BASE_KNOCKBACK,
                damage: SCALE_BASE_DMG,
            },
            Name::new("Scale"),
        ))
        .id();

    commands.entity(player).add_child(scale);

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
        Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
        Attack,
        PlayerProjectile,
        SpellType::Scale,
        Speed(config.speed),
        Knockback(config.knockback),
        Damage(config.damage),
        Direction(direction),
        Name::new("ScaleProjectile"),
    ));

    Ok(())
}

fn scale_hit(trigger: Trigger<ScaleHitEvent>, mut commands: Commands) {
    let enemy_entity = trigger.enemy;
    let projectile_entity = trigger.projectile;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy_entity,
        spell_entity: projectile_entity,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy_entity,
        spell_entity: projectile_entity,
    });

    commands.entity(projectile_entity).despawn();
}
