use avian2d::prelude::{CollisionStart, Collisions};
use bevy::prelude::*;

use crate::{
    PausableSystems,
    gameplay::{enemy::Enemy, weapons::prelude::*},
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_added_cast_weapon);
    app.add_systems(
        FixedUpdate,
        (tick_damage)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

/// Setup generic projectile observers
fn on_added_cast_weapon(
    event: On<Add, CastWeapon>,
    weapons: Query<Entity, With<CollisionDamage>>,
    projectile_q: Query<&CastWeapon>,
    mut commands: Commands,
) -> Result {
    let projectile = event.entity;
    let cast_weapon = projectile_q.get(projectile)?;

    let weapon = cast_weapon.0;
    if weapons.get(weapon).is_ok() {
        commands.entity(projectile).observe(on_projectile_collision);
    }

    Ok(())
}

pub fn on_projectile_collision(
    event: On<CollisionStart>,
    projectile_q: Query<&CastWeapon>,
    weapon_q: Query<(
        &HitSpec,
        &BaseDamage,
        Option<&ExplosionRadius>,
        Option<&DeathOnCollision>,
    )>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let projectile = event.collider1;
    let target = event.collider2;

    // Get weapon and damage mode
    let cast_weapon = projectile_q.get(projectile)?;
    let weapon = cast_weapon.0;

    let enemy_tf = enemy_q.get(target)?;

    let (hit, dmg, explosion_radius, death_on_collision) = weapon_q.get(weapon)?;

    trigger_hit_event(
        &mut commands,
        weapon,
        target,
        enemy_tf,
        hit,
        dmg,
        explosion_radius,
    );

    if death_on_collision.is_some() {
        commands.entity(projectile).despawn();
    }

    Ok(())
}

//NOTE: Currently we tick per weapon. That means switching enemies mid-tick is not possible.
fn tick_damage(
    projectiles: Query<&WeaponProjectiles>,
    mut weapons: Query<(
        Entity,
        &HitSpec,
        &BaseDamage,
        Option<&ExplosionRadius>,
        &mut TickDamage,
    )>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    collisions: Collisions,
    time: Res<Time>,
) {
    for (weapon, hit, dmg, explosion_radius, mut tick_timer) in &mut weapons {
        for projectile in projectiles.iter_descendants(weapon) {
            tick_timer.0.tick(time.delta());
            if tick_timer.0.just_finished() {
                for contact in collisions.entities_colliding_with(projectile) {
                    if let Ok(enemy_tf) = enemy_q.get(contact) {
                        trigger_hit_event(
                            &mut commands,
                            projectile,
                            contact,
                            enemy_tf,
                            hit,
                            dmg,
                            explosion_radius,
                        );
                    }
                }
            }
        }
    }
}

fn trigger_hit_event(
    commands: &mut Commands,
    weapon: Entity,
    target: Entity,
    enemy_tf: &Transform,
    hit_spec: &HitSpec,
    base_damage: &BaseDamage,
    explosion_radius: Option<&ExplosionRadius>,
) {
    commands.trigger(WeaponHitEvent {
        entity: weapon,
        target,
        hit_pos: enemy_tf.translation,
        dmg: base_damage.0,
        damage_type: hit_spec.damage_type,
        aoe: explosion_radius.map(|er| er.0),
        effects: hit_spec.effects.clone(),
    });
}
