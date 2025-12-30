use crate::audio::SfxPool;
use crate::gameplay::{
    damage_numbers::DamageType,
    enemy::{Enemy, EnemyDamageEvent, Root},
    simple_animation::AnimationPlayback,
    weapons::{
        behaviours::{WeaponImpactSfx, WeaponImpactVisuals},
        components::DoT,
        spec::components::OnHitEffect,
    },
};
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_projectile_hit_fx_sfx);
    app.add_observer(on_resolved_hit_damage);
    app.add_observer(on_resolved_hit_aoe);
    app.add_observer(on_resolved_hit_effects);
}

/// Emmitted when a weapon projectile hits a target
#[derive(EntityEvent, Clone, Debug)]
pub struct WeaponHitEvent {
    /// Weapon entity that fired the projectile
    pub entity: Entity,
    /// Target entity that was hit
    pub target: Entity,
    /// Position where the hit occurred
    pub hit_pos: Vec3,
    /// Damage dealt by the hit
    pub dmg: f32,
    /// Type of damage dealt
    pub damage_type: DamageType,
    /// Area of Effect radius, if applicable
    pub aoe: Option<f32>,
    /// Additional effects applied on hit
    pub effects: Vec<OnHitEffect>,
}

pub fn on_projectile_hit_fx_sfx(
    trigger: On<WeaponHitEvent>,
    weapon_q: Query<(Option<&WeaponImpactVisuals>, Option<&WeaponImpactSfx>)>,
    mut commands: Commands,
) -> Result {
    let ev = trigger.event();

    let (impact_vfx, impact_sfx) = weapon_q
        .get(ev.entity)
        .expect("ProjectileHitEvent fired for non-weapon entity");

    if let Some(vfx) = impact_vfx {
        let mut e = commands.spawn((
            Name::new("Impact VFX"),
            Transform::from_translation(ev.hit_pos),
            GlobalTransform::default(),
        ));

        vfx.0.apply_ec(&mut e);
        e.insert((AnimationPlayback::OnceDespawn,));
    }

    if let Some(sfx) = impact_sfx {
        commands.spawn((SamplePlayer::new(sfx.0.clone()), SfxPool));
    }

    Ok(())
}

pub fn on_resolved_hit_damage(trigger: On<WeaponHitEvent>, mut commands: Commands) -> Result {
    let ev = trigger.event();

    commands.trigger(EnemyDamageEvent {
        entity_hit: ev.target,
        dmg: ev.dmg,
        damage_type: ev.damage_type,
    });

    Ok(())
}

pub fn on_resolved_hit_aoe(
    trigger: On<WeaponHitEvent>,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let Some(r) = ev.aoe else {
        return;
    };

    for (other_e, other_tf) in &enemy_q {
        if other_e == ev.target {
            continue;
        }

        let dist = ev
            .hit_pos
            .truncate()
            .distance(other_tf.translation.truncate());

        if dist < r {
            commands.trigger(EnemyDamageEvent {
                entity_hit: other_e,
                dmg: ev.dmg,
                damage_type: ev.damage_type,
            });
        }
    }
}

pub fn on_resolved_hit_effects(
    trigger: On<WeaponHitEvent>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let ev = trigger.event();

    let enemy = enemy_q.get(ev.target)?;

    for eff in &ev.effects {
        match eff {
            OnHitEffect::Bleed {
                dps,
                duration,
                tick,
            } => {
                commands.entity(enemy).insert_if_new(DoT {
                    duration: Timer::from_seconds(*duration, TimerMode::Once),
                    tick: Timer::from_seconds(*tick, TimerMode::Once),
                    dmg_per_tick: *dps,
                    damage_type: DamageType::Bleed,
                });
            }
            OnHitEffect::Root { duration } => {
                commands
                    .entity(enemy)
                    .insert_if_new(Root(Timer::from_seconds(*duration, TimerMode::Once)));
            }
        }
    }

    Ok(())
}
