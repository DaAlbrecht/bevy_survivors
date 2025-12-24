use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// mod attack;
// mod movement;
mod setup;

pub(super) fn plugin(_app: &mut App) {
    // app.add_observer(attack::on_zone_attack);
}

#[derive(Component)]
pub struct ZoneAttack;

#[derive(Component, Reflect)]
pub struct ZoneBeam;

#[derive(Component, Reflect)]
pub struct ZoneCone {
    pub range: f32,
    pub angle: f32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneSpec {
    /// Duration the zone/beam stays active
    pub duration: f32,
    /// Width of the beam/zone (height is calculated based on target distance)
    pub width: f32,
    /// Whether the zone follows the player (true for beam/breath, false for arrow rain)
    pub follow_player: bool,
    /// Optional cone configuration (range and spread angle in degrees)
    #[serde(default)]
    pub cone: Option<ConeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ConeConfig {
    pub range: f32,
    pub angle_degrees: f32,
}
