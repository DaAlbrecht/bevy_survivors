use crate::gameplay::enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent};
use crate::gameplay::simple_animation::AnimationPlayback;
use crate::gameplay::{damage_numbers::DamageType, ws::assets::spec::OnHitEffect};
use crate::{
    audio::SfxPool,
    gameplay::ws::runtime::{sfx::WeaponImpactSfx, visuals::WeaponImpactVisuals},
};
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_projectile_hit_fx_sfx);
    app.add_observer(on_resolved_hit_damage);
    app.add_observer(on_resolved_hit_aoe);
    app.add_observer(on_resolved_hit_effects);
}

#[derive(EntityEvent, Clone, Debug)]
pub struct WeaponHitEvent {
    pub entity: Entity,
    pub projectile: Entity,
    pub target: Entity,
    pub hit_pos: Vec3,

    pub dmg: f32,
    pub damage_type: DamageType,
    pub aoe: Option<f32>,
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
            AnimationPlayback::OnceDespawn,
        ));

        vfx.0.apply_ec(&mut e);
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
) -> Result {
    let ev = trigger.event();
    let Some(r) = ev.aoe else {
        return Ok(());
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

    Ok(())
}

pub fn on_resolved_hit_effects(trigger: On<WeaponHitEvent>, mut commands: Commands) -> Result {
    let ev = trigger.event();

    for eff in &ev.effects {
        match eff {
            OnHitEffect::Knockback { .. } => {
                commands.trigger(EnemyKnockbackEvent {
                    entity_hit: ev.target,
                    projectile: ev.projectile,
                });
            }
            OnHitEffect::Bleed {
                dps,
                duration,
                tick,
            } => {
                todo!("Add Bleed Component")
            }
            OnHitEffect::Root { duration } => {
                todo!("Add Root Component")
            }
        }
    }

    Ok(())
}
