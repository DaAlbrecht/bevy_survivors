use bevy::prelude::*;

use crate::{
    ENEMY_SIZE, SPELL_SIZE,
    gameplay::{
        attacks::{
            fireball::{FireballAttackEvent, FireballHitEvent},
            lightning::LightningAttackEvent,
            orbs::OrbAttackEvent,
            scale::{ScaleAttackEvent, ScaleHitEvent},
        },
        enemy::{Enemy, Speed},
        player::{Direction, Inventory, Player},
    },
    screens::Screen,
};

pub mod fireball;
pub mod lightning;
pub mod orbs;
pub mod scale;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        scale::plugin,
        fireball::plugin,
        lightning::plugin,
        orbs::plugin,
    ));

    app.add_systems(
        Update,
        (attack, update_attack_timers, projectile_hit_detection).run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(FixedUpdate, move_projectile);
}

#[derive(Component, Default)]
pub(crate) struct Attack;

#[derive(Component)]
pub(crate) struct PlayerProjectile;

#[derive(Component, Default)]
pub(crate) struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Knockback(pub f32);

#[derive(Component)]
pub(crate) struct Damage(pub f32);

#[derive(Component)]
pub(crate) struct Range(pub f32);

#[derive(Component)]
pub(crate) struct ExplosionRadius(pub f32);

#[derive(Component)]
pub(crate) struct SpellDuration(pub Timer);

#[derive(Component)]
pub(crate) struct ProjectileCount(pub f32);

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) enum SpellType {
    Scale,
    Fireball,
    Lightning,
    Orb,
}

#[derive(Component, Default)]
pub(crate) struct Spell;

#[derive(Component)]
#[relationship(relationship_target = SpellProjectiles)]
struct CastSpell(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CastSpell, linked_spawn)]
struct SpellProjectiles(Vec<Entity>);

fn attack(
    player: Query<Entity, With<Player>>,
    inventory: Query<&Inventory>,
    mut spells: Query<(&mut Cooldown, &SpellType), With<Spell>>,
    mut commands: Commands,
) -> Result {
    let player = player.single()?;
    for inventory_slot in inventory.iter_descendants(player) {
        let (mut cooldown, spell_type) = spells.get_mut(inventory_slot)?;

        if cooldown.0.finished() {
            match spell_type {
                SpellType::Scale => commands.trigger(ScaleAttackEvent),
                SpellType::Fireball => commands.trigger(FireballAttackEvent),
                SpellType::Lightning => commands.trigger(LightningAttackEvent),
                SpellType::Orb => commands.trigger(OrbAttackEvent),
            }
            cooldown.0.reset();
        }
    }

    Ok(())
}

fn move_projectile(
    spells: Query<(Entity, &Speed), With<Spell>>,
    projectiles: Query<&SpellProjectiles>,
    mut projectile_q: Query<(&mut Transform, &Direction), With<PlayerProjectile>>,
    time: Res<Time>,
) -> Result {
    // Loop over all types of spells
    for (spell, speed) in &spells {
        // Iterate over each projectile for this given spell type
        for projectile in projectiles.iter_descendants(spell) {
            let (mut bullet_pos, bullet_direction) = projectile_q.get_mut(projectile)?;
            let movement = bullet_direction.0 * speed.0 * time.delta_secs();
            bullet_pos.translation += movement;
        }
    }
    Ok(())
}

fn projectile_hit_detection(
    spells: Query<(Entity, &SpellType), With<Spell>>,
    projectiles: Query<&SpellProjectiles>,
    enemy_q: Query<(&Transform, Entity), (With<Enemy>, Without<PlayerProjectile>)>,
    projectile_transform: Query<&Transform, With<PlayerProjectile>>,
    mut commands: Commands,
) -> Result {
    // Get all spells
    for (spell, spell_type) in &spells {
        // Get each fired projectile for this spell
        for projectile in projectiles.iter_descendants(spell) {
            // Get the position of this particular projectile
            let projectile_pos = projectile_transform.get(projectile)?;

            // Loop over all the positions of the enemies and check if one matches the position of
            // the projectile.
            for (&enemy_pos, enemy_entity) in &enemy_q {
                if (projectile_pos.translation.distance(enemy_pos.translation) - (SPELL_SIZE / 2.0))
                    <= ENEMY_SIZE / 2.0
                {
                    trigger_hit_event(&mut commands, spell_type, projectile, enemy_entity);
                }
            }
        }
    }
    Ok(())
}

fn update_attack_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut Cooldown, With<Spell>>,
    mut durations: Query<&mut SpellDuration, With<Spell>>,
) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }
}

pub(crate) fn trigger_hit_event(
    commands: &mut Commands,
    spell_type: &SpellType,
    projectile: Entity,
    enemy: Entity,
) {
    match spell_type {
        SpellType::Scale => commands.trigger(ScaleHitEvent { enemy, projectile }),
        SpellType::Fireball => commands.trigger(FireballHitEvent { enemy, projectile }),
        _ => {}
    }
}
