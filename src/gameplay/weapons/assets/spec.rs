use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;
use serde::{Deserialize, Serialize};

use crate::gameplay::{
    damage_numbers::DamageType,
    simple_animation::{AnimationIndices, AnimationTimer},
    weapons::{
        behaviours::{
            WeaponAttackSfx, WeaponImpactSfx, chain::ChainSpec, falling::FallingSpec,
            homing::HomingSpec, nova::NovaSpec, orbiters::OrbitersSpec, shot::ShotSpec,
            zone::ZoneSpec,
        },
        kind::WeaponKind,
    },
};

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

impl EntityCommand for HitSpec {
    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert(self);
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

impl EntityCommand for WeaponSfx {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(handle) = self.attack {
            entity.insert(WeaponAttackSfx(handle));
        }
        if let Some(handle) = self.impact {
            entity.insert(WeaponImpactSfx(handle));
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

impl EntityCommand for AttackSpec {
    fn apply(self, entity: EntityWorldMut) {
        match self {
            AttackSpec::Orbiters(s) => s.apply(entity),
            AttackSpec::Chain(s) => s.apply(entity),
            AttackSpec::Shot(s) => s.apply(entity),
            AttackSpec::Nova(s) => s.apply(entity),
            AttackSpec::Homing(s) => s.apply(entity),
            AttackSpec::Falling(s) => s.apply(entity),
            AttackSpec::Zone(spec) => spec.apply(entity),
        }
    }
}
