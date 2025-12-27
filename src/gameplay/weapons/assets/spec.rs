use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;
use serde::{Deserialize, Serialize};

use crate::gameplay::damage_numbers::DamageType;
use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::gameplay::weapons::ApplySpec;
use crate::gameplay::weapons::behaviours::chain::ChainSpec;
use crate::gameplay::weapons::behaviours::falling::FallingSpec;
use crate::gameplay::weapons::behaviours::homing::HomingSpec;
use crate::gameplay::weapons::behaviours::nova::NovaSpec;
use crate::gameplay::weapons::behaviours::orbiters::OrbitersSpec;
use crate::gameplay::weapons::behaviours::shot::ShotSpec;
use crate::gameplay::weapons::behaviours::zone::ZoneSpec;
use crate::gameplay::weapons::behaviours::{WeaponAttackSfx, WeaponImpactSfx};
use crate::gameplay::weapons::kind::WeaponKind;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct WeaponSpec {
    pub kind: WeaponKind,
    pub base_damage: f32,
    pub cooldown: f32,
    pub dot: Option<f32>,

    pub attack: AttackSpec,
    pub on_hit: HitSpec,

    pub visuals: VisualSpec,
    pub impact_visuals: Option<VisualSpec>,
    pub sfx: WeaponSfx,
    pub icon: Handle<Image>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OnHitEffect {
    Bleed { dps: f32, duration: f32, tick: f32 },
    Root { duration: f32 },
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HitSpec {
    pub damage_type: DamageType,
    pub effects: Vec<OnHitEffect>,
    pub knockback_strength: f32,
}

impl ApplySpec for HitSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity).insert(self.clone());
    }
}

#[derive(Debug, Clone)]
pub struct VisualSpec {
    pub image: Handle<Image>,
    pub size: Vec2,
    pub atlas: Option<AtlasAnimation>,
}

impl VisualSpec {
    pub fn apply_ec(&self, ec: &mut EntityCommands) {
        if let Some(atlas) = &self.atlas {
            ec.insert((
                Sprite::from_atlas_image(
                    self.image.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: atlas.first,
                    },
                ),
                AnimationIndices {
                    first: atlas.first,
                    last: atlas.last,
                },
                AnimationTimer::from_fps(atlas.fps),
            ));
        } else {
            ec.insert(Sprite {
                image: self.image.clone(),
                custom_size: Some(self.size),
                ..default()
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeaponSfx {
    pub attack: Option<Handle<AudioSample>>,
    pub impact: Option<Handle<AudioSample>>,
}

impl ApplySpec for WeaponSfx {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        if let Some(h) = &self.attack {
            ec.insert(WeaponAttackSfx(h.clone()));
        }
        if let Some(h) = &self.impact {
            ec.insert(WeaponImpactSfx(h.clone()));
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtlasAnimation {
    pub layout: Handle<TextureAtlasLayout>,
    pub first: usize,
    pub last: usize,
    pub fps: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AttackSpec {
    Orbiters(OrbitersSpec),
    Chain(ChainSpec),
    Shot(ShotSpec),
    Nova(NovaSpec),
    Homing(HomingSpec),
    Falling(FallingSpec),
    Zone(ZoneSpec),
}

impl ApplySpec for AttackSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        match self {
            AttackSpec::Orbiters(s) => s.apply(commands, entity),
            AttackSpec::Chain(s) => s.apply(commands, entity),
            AttackSpec::Shot(s) => s.apply(commands, entity),
            AttackSpec::Nova(s) => s.apply(commands, entity),
            AttackSpec::Homing(s) => s.apply(commands, entity),
            AttackSpec::Falling(s) => s.apply(commands, entity),
            AttackSpec::Zone(spec) => spec.apply(commands, entity),
        }
    }
}
