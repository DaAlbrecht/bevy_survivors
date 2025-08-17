use core::f32;
use std::f32::consts::PI;

use bevy::prelude::*;

use crate::gameplay::{
    attacks::{
        CastSpell, Cooldown, Damage, Knockback, PlayerProjectile, ProjectileCount, Range, Spell,
        SpellDuration, SpellType,
    },
    enemy::Speed,
    player::{AddToInventory, Direction, Player, spawn_player},
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Orb,
    Cooldown(Timer::from_seconds(4., TimerMode::Once)),
    SpellDuration(Timer::from_seconds(2., TimerMode::Once)),
    Range(75.),
    Speed(100.),
    Damage(1.),
    Knockback(750.),
    ProjectileCount(5.),
    Name::new("Orb Spell")
)]
pub(crate) struct Orb;

#[derive(Component)]
pub(crate) struct OrbProjectile;

#[derive(Event)]
pub(crate) struct OrbAttackEvent;

#[derive(Event)]
pub(crate) struct OrbHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, add_orb_spell.after(spawn_player));
    app.add_systems(Update, (update_orb_direction, orb_lifetime));
    app.add_observer(spawn_orb_projectile);
}

fn add_orb_spell(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    commands.spawn((Orb, AddToInventory(player)));

    Ok(())
}

fn spawn_orb_projectile(
    _trigger: Trigger<OrbAttackEvent>,
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
