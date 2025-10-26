use core::f32;
use std::f32::consts::PI;

use bevy::prelude::*;

use crate::gameplay::{
    Speed,
    enemy::{EnemyDamageEvent, EnemyKnockbackEvent},
    player::{Direction, Player},
    spells::{
        CastSpell, Cooldown, Damage, Knockback, Orbiting, PlayerProjectile, ProjectileCount, Range,
        Spell, SpellDuration, SpellType,
    },
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Orb,
    Cooldown(Timer::from_seconds(4., TimerMode::Once)),
    Orbiting,
    // SpellDuration(Timer::from_seconds(2., TimerMode::Once)),
    Range(75.),
    Speed(100.),
    Damage(1.),
    Knockback(750.),
    ProjectileCount(3.),
    Name::new("Orb Spell")
)]
#[derive(Reflect)]
pub(crate) struct Orb;

#[derive(Component, Reflect)]
pub(crate) struct OrbProjectile;

#[derive(Event, Reflect)]
pub(crate) struct OrbAttackEvent;

#[derive(Event, Reflect)]
pub(crate) struct OrbHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, (update_orb_direction, orb_lifetime));
    app.add_observer(spawn_orb_projectile);
    app.add_observer(orb_hit);
}

fn spawn_orb_projectile(
    _trigger: On<OrbAttackEvent>,
    player_q: Query<Entity, With<Player>>,
    orb_q: Query<(Entity, &Range, &ProjectileCount), With<Orb>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player = player_q.single()?;
    let (orb, radius, projectile_count) = orb_q.single()?;

    let mut pos_x: f32;
    let mut pos_y: f32;

    for n in 1..=projectile_count.0 as usize {
        let angle = (2.0 * PI) * (n as f32 / projectile_count.0);
        pos_x = f32::cos(angle);
        pos_y = f32::sin(angle);

        let orb_pos = Vec2::new(pos_x, pos_y) * radius.0;
        // V1(x,y) and V2(-y,x) are allways orthogonal and v2 looks counterclockwise
        let direction = Vec3::new(-orb_pos.y, orb_pos.x, 0.0).normalize();

        let orb = commands
            .spawn((
                Range(radius.0),
                Name::new("orb projectile"),
                Sprite {
                    image: asset_server.load("Orb.png"),
                    ..default()
                },
                OrbProjectile,
                CastSpell(orb),
                Transform::from_xyz(orb_pos.x, orb_pos.y, 0.),
                Direction(direction),
                PlayerProjectile,
                SpellDuration(Timer::from_seconds(4., TimerMode::Once)),
            ))
            .id();

        commands.entity(player).add_child(orb);
    }

    Ok(())
}

//Keeps direction orthogonal to radius -> circel
fn update_orb_direction(
    mut orb_q: Query<(&mut Transform, &mut Direction, &Range), With<OrbProjectile>>,
) {
    for (mut orb_pos, mut direction, orbit_radius) in &mut orb_q {
        let mut pos_vec = orb_pos.translation.truncate();

        //Clamp the orb onto the circle radius.
        let radius = orbit_radius.0.max(0.001); // avoid divide-by-zero

        let length = pos_vec.length();
        if length == 0.0 {
            pos_vec = Vec2 { x: radius, y: 0.0 };
        } else {
            //scale factor for radius
            let k = radius / length;
            pos_vec *= k;
        }

        orb_pos.translation = pos_vec.extend(0.0);

        direction.0 = Vec3::new(-pos_vec.y, pos_vec.x, 0.0).normalize();

        // info!("Transfrom: {:?}", orb_pos);
    }
}

fn orb_hit(
    trigger: On<OrbHitEvent>,
    mut commands: Commands,
    orb_dmg: Query<&Damage, With<Orb>>,
) -> Result {
    let enemy = trigger.enemy;
    let spell_entity = trigger.projectile;
    let dmg = orb_dmg.single()?.0;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy,
        dmg,
    });

    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy,
        spell_entity,
    });

    Ok(())
}

fn orb_lifetime(
    mut commands: Commands,
    mut orb_q: Query<(Entity, &mut SpellDuration), With<OrbProjectile>>,
) {
    for (orb, duration) in &mut orb_q {
        if duration.0.is_finished() {
            commands.entity(orb).despawn();
        }
    }
}
