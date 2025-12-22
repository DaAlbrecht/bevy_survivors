use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;
use serde::{Deserialize, Serialize};

use crate::gameplay::damage_numbers::DamageType;
use crate::gameplay::ws::prelude::*;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct WeaponSpec {
    pub kind: WeaponKind,
    pub base_damage: f32,
    pub cooldown: f32,

    pub attack: AttackSpec,
    pub on_hit: HitSpec,

    pub visuals: VisualSpec,
    pub impact_visuals: Option<VisualSpec>,
    pub sfx: WeaponSfx,
    pub icon: Handle<Image>,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HitSpec {
    pub damage_type: DamageType,
    pub effects: Vec<OnHitEffect>,
    pub knockback_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OnHitEffect {
    Bleed { dps: f32, duration: f32, tick: f32 },
    Root { duration: f32 },
}

#[derive(Debug, Clone)]
pub struct VisualSpec {
    pub image: Handle<Image>,
    pub size: Vec2,
    pub atlas: Option<AtlasAnim>,
}

#[derive(Debug, Clone)]
pub struct AtlasAnim {
    pub layout: Handle<TextureAtlasLayout>,
    pub first: usize,
    pub last: usize,
    pub fps: u8,
}

#[derive(Debug, Clone)]
pub struct WeaponSfx {
    pub attack: Option<Handle<AudioSample>>,
    pub impact: Option<Handle<AudioSample>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AttackSpec {
    Orbiters(OrbitersSpec),
    ChainLightning(ChainLightningSpec),
    Shot(ShotSpec),
    Nova(NovaSpec),
    Homing(HomingSpec),
    Falling(FallingSpec),
}
