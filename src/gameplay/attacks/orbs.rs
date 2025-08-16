use core::f32;
use std::f32::consts::PI;

use bevy::prelude::*;

use crate::gameplay::{
    attacks::{
        Attack, Cooldown, Damage, Knockback, PlayerProjectile, ProjectileConfig, Range,
        SpellDuration, SpellType,
    },
    enemy::Speed,
    player::{Direction, Player, spawn_player},
};

const ORB_BASE_COOLDOWN: f32 = 4.0;
const ORB_BASE_DURATION: f32 = 2.0;
const ORB_BASE_DAMAGE: f32 = 1.0;
const ORB_BASE_KNOCKBACK: f32 = 750.0;
const ORB_BASE_SPEED: f32 = 100.0;
const ORB_BASE_COUNT: f32 = 5.0;
const ORB_BASE_RANGE: f32 = 75.0;

#[derive(Component)]
pub(crate) struct Orb;

#[derive(Component)]
pub(crate) struct OrbProjectile;

//angular speed
#[derive(Component)]
pub(crate) struct OrbSpeed(pub f32);

#[derive(Event)]
pub(crate) struct OrbAttackEvent;

#[derive(Event)]
pub(crate) struct OrbHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

#[derive(Component)]
pub(crate) struct OrbCount(pub f32);

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_orb.after(spawn_player));
    app.add_systems(Update, (update_orb_direction, orb_lifetime));
    app.add_observer(spawn_orb_projectile);
}

fn spawn_orb(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    let orb = commands
        .spawn((
            Attack,
            Orb,
            SpellType::Orb,
            Cooldown(Timer::from_seconds(ORB_BASE_COOLDOWN, TimerMode::Once)),
            SpellDuration(Timer::from_seconds(ORB_BASE_DURATION, TimerMode::Once)),
            Range(ORB_BASE_RANGE),
            OrbCount(ORB_BASE_COUNT),
            ProjectileConfig {
                speed: ORB_BASE_SPEED,
                damage: ORB_BASE_DAMAGE,
                knockback: ORB_BASE_KNOCKBACK,
            },
        ))
        .id();

    commands.entity(player).add_child(orb);

    Ok(())
}

fn spawn_orb_projectile(
    _trigger: Trigger<OrbAttackEvent>,
    player_q: Query<Entity, With<Player>>,
    orb_q: Query<(&ProjectileConfig, &Range, &OrbCount), With<Orb>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player = player_q.single()?;
    let (config, radius, count) = orb_q.single()?;

    let mut pos_x: f32;
    let mut pos_y: f32;

    for n in 1..=count.0 as usize {
        let angle = (2.0 * PI) * (n as f32 / count.0);
        pos_x = f32::cos(angle);
        pos_y = f32::sin(angle);

        let orb_pos = Vec2::new(pos_x, pos_y) * radius.0;
        // V1(x,y) and V2(-y,x) are allways orthogonal and v2 looks counterclockwise
        let direction = Vec3::new(-orb_pos.y, orb_pos.x, 0.0).normalize();

        let orb = commands
            .spawn((
                Sprite {
                    image: asset_server.load("Orb.png"),
                    ..default()
                },
                Transform::from_xyz(orb_pos.x, orb_pos.y, 0.0),
                Attack,
                PlayerProjectile,
                OrbProjectile,
                SpellType::Orb,
                Range(radius.0),
                Speed(config.speed),
                Knockback(config.knockback),
                Damage(config.damage),
                Direction(direction),
                Name::new("OrbProjectile"),
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
    }
}

fn orb_lifetime(
    mut commands: Commands,
    mut orb_q: Query<&mut SpellDuration, With<Orb>>,
    projectile_q: Query<Entity, With<OrbProjectile>>,
) -> Result {
    let mut orb_duration = orb_q.single_mut()?;

    if orb_duration.0.finished() {
        for orb in projectile_q {
            commands.entity(orb).despawn();
        }
        orb_duration.0.reset();
    }

    Ok(())
}
