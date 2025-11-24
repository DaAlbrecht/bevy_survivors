use avian2d::prelude::CollisionStart;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::gameplay::damage_numbers::DamageType;
use crate::gameplay::spells::UpgradeSpellEvent;
use crate::gameplay::{
    Speed,
    enemy::{EnemyDamageEvent, EnemyKnockbackEvent},
    player::{Direction, Player},
    spells::{CastSpell, Damage, Knockback, PlayerProjectile, Spell, SpellType},
};

use super::Cooldown;

use bevy_rand::{global::GlobalRng, prelude::WyRand};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Scale,
    Cooldown(Timer::from_seconds(1., TimerMode::Once)),
    Speed(50.),
    Knockback(1500.),
    Damage(5.),
    Name::new("Scale")
)]
#[derive(Reflect)]
pub(crate) struct Scale;

#[derive(Event, Reflect)]
pub(crate) struct ScaleAttackEvent;

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(spawn_scale_projectile);
}

pub fn upgrade_scale(
    _trigger: On<UpgradeSpellEvent>,
    mut scale_q: Query<&mut Knockback, With<Scale>>,
) -> Result {
    let mut knockback = scale_q.single_mut()?;
    knockback.0 += 100.0;
    info!("Scale knockback upgraded to: {}", knockback.0);

    Ok(())
}

fn spawn_scale_projectile(
    _trigger: On<ScaleAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    scale: Query<Entity, With<Scale>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let scale = scale.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

    commands
        .spawn((
            Name::new("scale projectile"),
            Sprite {
                image: asset_server.load("fx/scale.png"),
                ..default()
            },
            CastSpell(scale),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.),
            Direction(direction),
            PlayerProjectile,
        ))
        .observe(on_scale_hit);

    Ok(())
}

fn on_scale_hit(
    // trigger: On<ScaleHitEvent>,
    event: On<CollisionStart>,
    mut commands: Commands,
    scale_dmg: Query<&Damage, With<Scale>>,
) -> Result {
    info!("hit detected");
    let spell = event.collider1;
    let enemy = event.collider2;

    let dmg = scale_dmg.single()?.0;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy,
        dmg,
        damage_type: DamageType::Physical,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy,
        spell_entity: spell,
    });

    commands.entity(spell).despawn();
    Ok(())
}
