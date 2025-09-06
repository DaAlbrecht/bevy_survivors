use bevy::prelude::*;

use crate::{
    ENEMY_SIZE, SPELL_SIZE,
    gameplay::{
        PickUpSpell,
        enemy::{DamageCooldown, Enemy, EnemyDamageEvent, Speed},
        player::{AddToInventory, Direction, Inventory, Player},
        spells::{
            dot::Bleed,
            fireball::{Fireball, FireballAttackEvent, FireballHitEvent},
            lightning::{Lightning, LightningAttackEvent},
            orbs::{Orb, OrbAttackEvent, OrbHitEvent},
            scale::{Scale, ScaleAttackEvent, ScaleHitEvent},
            thorn::{Thorn, ThornAttackEvent, ThornHitEvent},
        },
    },
    screens::Screen,
};

pub mod dot;
pub mod fireball;
pub mod lightning;
pub mod orbs;
pub mod scale;
pub mod thorn;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        scale::plugin,
        fireball::plugin,
        lightning::plugin,
        orbs::plugin,
        thorn::plugin,
        dot::plugin,
    ));

    app.add_systems(
        Update,
        (attack, handle_timers, projectile_hit_detection).run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(FixedUpdate, move_projectile);

    app.add_observer(add_spell_to_inventory);

    app.register_type::<SpellType>();
}

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

#[derive(Component)]
pub(crate) struct Halt;

#[derive(Component)]
pub(crate) struct StartPosition(Vec2);

#[derive(Component)]
pub(crate) struct Despawn;

#[derive(Component, Clone, Copy, PartialEq, Debug, Reflect)]
pub(crate) enum SpellType {
    Scale,
    Fireball,
    Lightning,
    Orb,
    Thorn,
}

impl SpellType {
    pub const ALL: [SpellType; 5] = [
        SpellType::Scale,
        SpellType::Fireball,
        SpellType::Lightning,
        SpellType::Orb,
        SpellType::Thorn,
    ];
}

#[derive(Component, Default)]
pub(crate) struct Spell;

#[derive(Component)]
#[relationship(relationship_target = SpellProjectiles)]
pub(crate) struct CastSpell(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CastSpell, linked_spawn)]
pub(crate) struct SpellProjectiles(Vec<Entity>);

#[derive(Component, Default)]
pub(crate) struct Orbiting;

#[derive(Component, Default)]
pub(crate) struct Segmented;

#[derive(Component)]
pub(crate) struct Root(pub Timer);

#[derive(Component)]
pub(crate) struct Tail;

pub(crate) fn add_spell_to_inventory(
    trigger: Trigger<PickUpSpell>,
    mut commands: Commands,
    player: Query<Entity, (With<Player>, Without<Spell>)>,
    owned_spells: Query<&SpellType, With<Spell>>,
) -> Result {
    for owned_spell in owned_spells {
        if *owned_spell == trigger.spell_type {
            //TODO: upgrade spell instead
            info!("spell_type already owned {:?}", owned_spell);
            return Ok(());
        }
    }

    let player = player.single()?;
    //Get Inventory of Player
    let mut e = commands.spawn(AddToInventory(player));

    match trigger.spell_type {
        SpellType::Scale => {
            e.insert(Scale);
        }
        SpellType::Fireball => {
            e.insert(Fireball);
        }
        SpellType::Lightning => {
            e.insert(Lightning);
        }
        SpellType::Orb => {
            e.insert(Orb);
        }
        SpellType::Thorn => {
            e.insert(Thorn);
        }
    }

    Ok(())
}

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
                SpellType::Thorn => commands.trigger(ThornAttackEvent),
            }
            cooldown.0.reset();
        }
    }

    Ok(())
}

fn move_projectile(
    spells: Query<(Entity, &Speed), With<Spell>>,
    projectiles: Query<&SpellProjectiles>,
    mut projectile_q: Query<(&mut Transform, &Direction), (With<PlayerProjectile>, Without<Halt>)>,
    time: Res<Time>,
) -> Result {
    // Loop over all types of spells
    for (spell, speed) in &spells {
        // Iterate over each projectile for this given spell type

        for projectile in projectiles.iter_descendants(spell) {
            let Ok((mut bullet_pos, bullet_direction)) = projectile_q.get_mut(projectile) else {
                continue;
            };

            let movement = bullet_direction.0 * speed.0 * time.delta_secs();
            bullet_pos.translation += movement;
        }
    }
    Ok(())
}

fn projectile_hit_detection(
    spells: Query<(Entity, &SpellType, Option<&Orbiting>), With<Spell>>,
    player_transform: Query<&Transform, With<Player>>,
    tail_transform: Query<&GlobalTransform, With<Tail>>,
    projectiles: Query<&SpellProjectiles>,
    enemy_q: Query<(&Transform, Entity), (With<Enemy>, Without<PlayerProjectile>)>,
    projectile_transform: Query<&Transform, With<PlayerProjectile>>,
    mut commands: Commands,
) -> Result {
    // Get all spells
    for (spell, spell_type, orbiting) in &spells {
        // Get each fired projectile for this spell
        for projectile in projectiles.iter_descendants(spell) {
            // Get the position of this particular projectile
            let mut projectile_pos = projectile_transform.get(projectile)?.translation;

            // If projectile is orbiting the player get gloabl pos
            if orbiting.is_some() {
                let player_pos = player_transform.single()?;
                projectile_pos += player_pos.translation;
            }

            //Get Globaltransform if projectile is a tail
            if tail_transform.get(projectile).is_ok() {
                projectile_pos = tail_transform.get(projectile)?.translation();
            }

            // Loop over all the positions of the enemies and check if one matches the position of
            // the projectile.
            for (enemy_pos, enemy_entity) in enemy_q {
                if (projectile_pos.distance(enemy_pos.translation) - (SPELL_SIZE / 2.0))
                    <= ENEMY_SIZE / 2.0
                {
                    trigger_hit_event(&mut commands, spell_type, projectile, enemy_entity);
                }
            }
        }
    }
    Ok(())
}

fn handle_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut Cooldown, With<Spell>>,
    mut durations: Query<&mut SpellDuration, With<PlayerProjectile>>,
    mut thorn_dmg_timer: Query<&mut DamageCooldown, With<Thorn>>,
    mut root_timer: Query<(Entity, &mut Root), With<Enemy>>,
    mut bleed_timer: Query<(Entity, &mut Bleed), With<Enemy>>,
    mut commands: Commands,
) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }

    for mut dmg_timer in &mut thorn_dmg_timer {
        dmg_timer.0.tick(time.delta());
    }

    for (entity, mut root) in &mut root_timer {
        root.0.tick(time.delta());
        if root.0.finished() {
            commands.entity(entity).remove::<Root>();
        }
    }

    for (target, mut bleed) in &mut bleed_timer {
        bleed.duration.tick(time.delta());
        bleed.tick.tick(time.delta());
        if bleed.tick.finished() {
            info!("Bleed deals dmg");
            commands.trigger(EnemyDamageEvent {
                entity_hit: target,
                dmg: bleed.dmg_per_tick,
            });
            bleed.tick.reset();
        }
        if bleed.duration.finished() {
            commands.entity(target).remove::<Bleed>();
        }
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
        SpellType::Orb => commands.trigger(OrbHitEvent { enemy, projectile }),
        SpellType::Thorn => commands.trigger(ThornHitEvent { enemy, projectile }),
        _ => (),
    }
}
