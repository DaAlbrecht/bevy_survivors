use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    GameLayer, PausableSystems,
    gameplay::{
        PickUpSpell, Speed,
        enemy::{DamageCooldown, Enemy, EnemyDamageEvent},
        player::{AddToInventory, Direction, Inventory, Player},
        spells::{
            dot::Bleed,
            fireball::{Fireball, FireballAttackEvent, upgrade_fireball},
            lightning::{Lightning, LightningAttackEvent, upgrade_lightning},
            orbs::{Orb, OrbAttackEvent, upgrade_orb},
            scale::{Scale, ScaleAttackEvent, upgrade_scale},
            thorn::{Thorn, ThornAttackEvent, upgrade_thorn},
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
        FixedUpdate,
        (handle_timers, attack, move_projectile)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_observer(add_spell_to_inventory);

    app.register_type::<SpellType>();
}

#[derive(Component, Reflect)]
#[require(
    RigidBody::Kinematic,
    Collider= Collider::rectangle(16., 16.) ,
    DebugRender = DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
    CollisionEventsEnabled,
    CollisionLayers =  CollisionLayers::new(
        GameLayer::Player,
        [
            GameLayer::Enemy,
            GameLayer::Default,
        ],
    ),
)]
pub(crate) struct PlayerProjectile;

#[derive(Component, Default, Reflect)]
pub(crate) struct Cooldown(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Knockback(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Damage(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Range(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct ExplosionRadius(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct SpellDuration(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct ProjectileCount(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Halt;

#[derive(Component, Reflect)]
pub(crate) struct StartPosition(Vec2);

#[derive(Component, Reflect)]
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

#[derive(Component, Default, Reflect)]
pub(crate) struct Spell;

#[derive(Component)]
#[relationship(relationship_target = SpellProjectiles)]
#[derive(Reflect)]
pub(crate) struct CastSpell(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CastSpell, linked_spawn)]
#[derive(Reflect)]
pub(crate) struct SpellProjectiles(Vec<Entity>);

#[derive(Component, Default, Reflect)]
pub(crate) struct Segmented;

#[derive(Component, Reflect)]
pub(crate) struct Root(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct Tail;

#[derive(Component)]
pub struct SpellTick(pub Timer);

#[derive(EntityEvent)]
pub(crate) struct UpgradeSpellEvent {
    pub entity: Entity,
}

pub(crate) fn add_spell_to_inventory(
    trigger: On<PickUpSpell>,
    mut commands: Commands,
    player_q: Query<Entity, (With<Player>, Without<Spell>)>,
    owned_spells: Query<(Entity, &SpellType), With<Spell>>,
) -> Result {
    let Ok(player) = player_q.single() else {
        return Ok(());
    };
    //
    // Check if spell is already owned - if so, upgrade it
    for (spell_entity, owned_spell) in &owned_spells {
        if *owned_spell == trigger.spell_type {
            info!("Upgrading spell: {:?}", owned_spell);
            // Trigger upgrade event on the spell entity itself
            commands.trigger(UpgradeSpellEvent {
                entity: spell_entity,
            });
            return Ok(());
        }
    }

    //Get Inventory of Player
    let mut e = commands.spawn(AddToInventory(player));

    match trigger.spell_type {
        SpellType::Scale => {
            e.insert(Scale);
            e.observe(upgrade_scale);
        }
        SpellType::Fireball => {
            e.insert(Fireball);
            e.observe(upgrade_fireball);
        }
        SpellType::Lightning => {
            e.insert(Lightning);
            e.observe(upgrade_lightning);
        }
        SpellType::Orb => {
            e.insert(Orb);
            e.observe(upgrade_orb);
        }
        SpellType::Thorn => {
            e.insert(Thorn);
            e.observe(upgrade_thorn);
        }
    }

    Ok(())
}

fn attack(
    player_q: Query<Entity, With<Player>>,
    inventory: Query<&Inventory>,
    mut spells: Query<(&mut Cooldown, &SpellType), With<Spell>>,
    mut commands: Commands,
) -> Result {
    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    for inventory_slot in inventory.iter_descendants(player) {
        let (mut cooldown, spell_type) = spells.get_mut(inventory_slot)?;

        if cooldown.0.is_finished() {
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

fn handle_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut Cooldown, With<Spell>>,
    mut durations: Query<&mut SpellDuration, With<CastSpell>>,
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
        if root.0.is_finished() {
            commands.entity(entity).remove::<Root>();
        }
    }

    for (target, mut bleed) in &mut bleed_timer {
        bleed.duration.tick(time.delta());
        bleed.tick.tick(time.delta());
        if bleed.tick.is_finished() {
            commands.trigger(EnemyDamageEvent {
                entity_hit: target,
                dmg: bleed.dmg_per_tick,
            });
            bleed.tick.reset();
        }
        if bleed.duration.is_finished() {
            commands.entity(target).remove::<Bleed>();
        }
    }
}

fn move_projectile(
    spells: Query<(Entity, &Speed), With<Spell>>,
    projectiles: Query<&SpellProjectiles>,
    mut projectile_q: Query<
        (&mut LinearVelocity, &Direction),
        (With<PlayerProjectile>, Without<Halt>),
    >,
) {
    // Loop over all types of spells
    for (spell, speed) in &spells {
        // Iterate over each projectile for this given spell type

        for projectile in projectiles.iter_descendants(spell) {
            let Ok((mut linear_velocity, bullet_direction)) = projectile_q.get_mut(projectile)
            else {
                continue;
            };

            let movement = bullet_direction.0.normalize_or_zero() * speed.0;
            linear_velocity.0.x = movement.x;
            linear_velocity.0.y = movement.y;
        }
    }
}
