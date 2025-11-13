use core::f32;

use bevy::prelude::*;

use crate::{
    PausableSystems, PhysicsAppSystems,
    gameplay::{
        enemy::{EnemyDamageEvent, EnemyKnockbackEvent},
        movement::{MovementController, PhysicalTranslation, PreviousPhysicalTranslation},
        player::Player,
        spells::{
            CastSpell, Cooldown, Damage, PlayerProjectile, ProjectileCount, Range, Spell,
            SpellDuration, SpellType, UpgradeSpellEvent,
        },
    },
    screens::Screen,
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Orb,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    // SpellDuration(Timer::from_seconds(2., TimerMode::Once)),
    Range(75.),
    Damage(1.),
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

#[derive(Component, Reflect)]
struct OrbPhase(pub f32);

// orbital angular speed (radians/sec). Tweak for orbit period.
const ORB_ANGULAR_SPEED: f32 = std::f32::consts::TAU * 0.25; // one orbit per 4s

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (record_orb_movement, orb_lifetime)
            .in_set(PhysicsAppSystems::PhysicsAdjustments)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(spawn_orb_projectile);
    app.add_observer(orb_hit);
    app.add_observer(upgrade_orb);
}

fn upgrade_orb(
    _trigger: On<UpgradeSpellEvent>,
    mut orb_q: Query<&mut ProjectileCount, With<Orb>>,
) -> Result {
    let mut count = orb_q.single_mut()?;
    count.0 *= 2.0;
    info!("Orb projectile count upgraded to: {}", count.0);

    Ok(())
}

fn spawn_orb_projectile(
    _trigger: On<OrbAttackEvent>,
    player: Query<&PhysicalTranslation, With<Player>>,
    orb_q: Query<(Entity, &Range, &ProjectileCount), With<Orb>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player.single()?;
    let (orb, radius, projectile_count) = orb_q.single()?;

    for n in 1..=projectile_count.0 as usize {
        // Compute starting phase for each orb (even spacing)
        let phase = (std::f32::consts::TAU) * (n as f32 / projectile_count.0);
        let offset = Vec2::from_angle(phase) * radius.0;
        let world_pos = player_pos.0 + offset.extend(0.0);

        // tangent direction (orthogonal to radius)
        let direction = Vec3::new(-offset.y, offset.x, 0.0).normalize();
        let tangential_speed = ORB_ANGULAR_SPEED * radius.0;

        commands.spawn((
            Name::new("orb projectile"),
            Sprite {
                image: asset_server.load("orb.png"),
                ..default()
            },
            OrbProjectile,
            CastSpell(orb),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            PreviousPhysicalTranslation(world_pos),
            PhysicalTranslation(world_pos),
            MovementController {
                velocity: direction,
                // set tangential speed so advance_physics will move the orb immediately
                speed: tangential_speed,
                mass: 20.0,
                ..default()
            },
            OrbPhase(phase),
            Range(radius.0),
            PlayerProjectile,
            SpellDuration(Timer::from_seconds(4., TimerMode::Once)),
        ));
    }

    Ok(())
}

//Keeps direction orthogonal to radius -> circel
fn record_orb_movement(
    player_q: Query<&PhysicalTranslation, (With<Player>, Without<OrbProjectile>)>,
    mut orb_q: Query<
        (
            &PhysicalTranslation,
            &mut MovementController,
            &mut OrbPhase,
            &Range,
        ),
        (With<OrbProjectile>, Without<Player>),
    >,
    time: Res<Time<Fixed>>,
) -> Result {
    let dt = time.delta_secs();
    let player_pos = player_q.single()?;

    for (orb_pos, mut controller, mut phase, orbit_radius) in &mut orb_q {
        // Advance orbital phase
        phase.0 += ORB_ANGULAR_SPEED * dt;
        if phase.0 > std::f32::consts::TAU {
            phase.0 -= std::f32::consts::TAU;
        }

        // Compute the target orbit position relative to player
        let offset = Vec2::from_angle(phase.0) * orbit_radius.0;
        let target_pos = player_pos.0 + offset.extend(0.0);

        // Compute velocity needed to reach target_pos this frame
        let delta = target_pos - orb_pos.0;
        let velocity = if dt > 0.0 { delta / dt } else { Vec3::ZERO };

        // Apply velocity to MovementController
        controller.velocity = velocity.normalize_or_zero();
        controller.speed = velocity.length();
    }

    Ok(())
}

fn orb_hit(
    trigger: On<OrbHitEvent>,
    mut commands: Commands,
    orb_dmg: Query<&Damage, With<Orb>>,
) -> Result {
    info!("Orb hit enemy: {:?}", trigger.enemy);
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
            info!("Despawning orb projectile: {:?}", orb);
            commands.entity(orb).despawn();
        }
    }
}
