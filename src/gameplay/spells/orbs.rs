use core::f32;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    GameLayer, PausableSystems,
    gameplay::{
        damage_numbers::DamageType,
        enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
        player::{Direction, Player},
        spells::{
            CastSpell, Cooldown, Damage, ProjectileCount, Range, Spell, SpellDuration, SpellType,
            UpgradeSpellEvent,
        },
    },
    screens::Screen,
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Orb,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    Range(75.),
    Damage(4.),
    ProjectileCount(3.),
    Name::new("Orb Spell")
)]
#[derive(Reflect)]
pub(crate) struct Orb;

#[derive(Component, Reflect)]
pub(crate) struct OrbProjectile;

#[derive(Event, Reflect)]
pub(crate) struct OrbAttackEvent;

#[derive(Component, Reflect)]
struct OrbPhase(pub f32);

// orbital angular speed (radians/sec). Tweak for orbit period.
const ORB_ANGULAR_SPEED: f32 = std::f32::consts::TAU * 0.25; // one orbit per 4s

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (update_orb_movement, orb_lifetime)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(spawn_orb_projectile);
}

pub fn upgrade_orb(
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
    player_q: Query<&Transform, With<Player>>,
    orb_q: Query<(Entity, &Range, &ProjectileCount), With<Orb>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(player_transform) = player_q.single() else {
        return Ok(());
    };

    let (orb, radius, projectile_count) = orb_q.single()?;

    for n in 1..=projectile_count.0 as usize {
        // Compute starting phase for each orb (even spacing)
        let phase = (std::f32::consts::TAU) * (n as f32 / projectile_count.0);
        let offset = Vec2::from_angle(phase) * radius.0;
        let world_pos = player_transform.translation + offset.extend(10.0);

        // tangent direction (orthogonal to radius)
        let direction = Vec3::new(-offset.y, offset.x, 0.0).normalize();

        commands
            .spawn((
                Name::new("orb projectile"),
                Sprite {
                    image: asset_server.load("fx/orb.png"),
                    ..default()
                },
                Collider::rectangle(16., 16.),
                DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
                TranslationInterpolation,
                CollisionEventsEnabled,
                CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
                OrbProjectile,
                CastSpell(orb),
                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                SpellDuration(Timer::from_seconds(4., TimerMode::Once)),
                OrbPhase(phase),
                Direction(direction),
                Range(radius.0),
            ))
            .observe(on_orb_hit);
    }

    Ok(())
}

fn on_orb_hit(
    event: On<CollisionStart>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut commands: Commands,
    spell_q: Query<&Damage, With<Orb>>,
) -> Result {
    let spell = event.collider1;
    let enemy = event.collider2;

    let dmg = spell_q.single()?;

    if let Ok(enemy) = enemy_q.get(enemy) {
        let dmg = dmg.0;

        commands.trigger(EnemyDamageEvent {
            entity_hit: enemy,
            dmg,
            damage_type: DamageType::Physical,
        });

        //Knockback
        commands.trigger(EnemyKnockbackEvent {
            entity_hit: enemy,
            spell_entity: spell,
        });
    }

    Ok(())
}

//Keeps direction orthogonal to radius -> circel
fn update_orb_movement(
    player_q: Query<&Transform, (With<Player>, Without<OrbProjectile>)>,
    mut orb_q: Query<
        (&mut Transform, &mut OrbPhase, &Range),
        (With<OrbProjectile>, Without<Player>),
    >,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_q.single() else {
        return;
    };

    for (mut orb_transform, mut phase, orbit_radius) in &mut orb_q {
        // Advance orbital phase
        phase.0 += ORB_ANGULAR_SPEED * dt;
        if phase.0 > std::f32::consts::TAU {
            phase.0 -= std::f32::consts::TAU;
        }

        // Compute the target orbit position relative to player
        let offset = Vec2::from_angle(phase.0) * orbit_radius.0;
        let target_pos = player_transform.translation + offset.extend(10.0);

        // Update orb position
        orb_transform.translation = target_pos;
    }
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
