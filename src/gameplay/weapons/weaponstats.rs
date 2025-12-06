use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct WeaponStats {
    pub level: f32,
    pub damage: f32,
    pub speed: f32,
    pub range: f32,
    pub explosion_radius: f32,
    pub knockback: f32,
    pub projectile_count: f32,
    pub max_hits: usize,
    pub cooldown: f32,
    pub lifetime: f32,
    pub damage_cooldown: f32,
    pub dot: DotStats,
}

#[derive(Default)]
pub(crate) struct DotStats {
    pub damage: f32,
    pub duration: f32,
    pub tick: f32,
}

#[derive(Resource)]
pub(crate) struct FireBallLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct EnergyLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct CirclesLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct IcelanceLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct LightningLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct ScaleLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct ThornLevels {
    pub levels: VecDeque<WeaponStats>,
}

#[derive(Resource)]
pub(crate) struct OrbLevels {
    pub levels: VecDeque<WeaponStats>,
}

pub(crate) fn make_fireball_levels() -> FireBallLevels {
    FireBallLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 5.,
                speed: 600.,
                range: 200.,
                explosion_radius: 50.,
                knockback: 100.,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 10.,
                speed: 600.,
                range: 200.,
                explosion_radius: 75.,
                knockback: 100.,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 15.,
                speed: 600.,
                range: 250.,
                explosion_radius: 100.,
                knockback: 100.,
                cooldown: 4.,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 20.,
                speed: 600.,
                range: 300.,
                explosion_radius: 125.,
                knockback: 150.,
                cooldown: 3.,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 30.,
                speed: 600.,
                range: 300.,
                explosion_radius: 150.,
                knockback: 150.,
                cooldown: 2.,
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_energy_levels() -> EnergyLevels {
    EnergyLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 5.,
                speed: 400.,
                cooldown: 5.,
                projectile_count: 1.,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 10.,
                speed: 400.,
                cooldown: 4.,
                projectile_count: 1.,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 15.,
                speed: 400.,
                cooldown: 4.,
                projectile_count: 2.,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 20.,
                speed: 400.,
                cooldown: 3.,
                projectile_count: 2.,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 30.,
                speed: 400.,
                cooldown: 3.,
                projectile_count: 3.,
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_circles_levels() -> CirclesLevels {
    CirclesLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 5.,
                cooldown: 5.,
                projectile_count: 4.,
                lifetime: 10.,
                max_hits: 5,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 10.,
                cooldown: 5.,
                projectile_count: 4.,
                lifetime: 10.,
                max_hits: 5,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 15.,
                cooldown: 5.,
                projectile_count: 4.,
                lifetime: 10.,
                max_hits: 5,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 20.,
                cooldown: 5.,
                projectile_count: 4.,
                lifetime: 10.,
                max_hits: 5,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 30.,
                cooldown: 5.,
                projectile_count: 4.,
                lifetime: 10.,
                max_hits: 5,
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_icelance_levels() -> IcelanceLevels {
    IcelanceLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 1.,
                speed: 400.,
                explosion_radius: 50.,
                knockback: 100.,
                cooldown: 0.5,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 2.,
                speed: 600.,
                explosion_radius: 75.,
                knockback: 100.,
                cooldown: 0.5,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 3.,
                speed: 600.,
                explosion_radius: 100.,
                knockback: 100.,
                cooldown: 0.5,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 4.,
                speed: 600.,
                explosion_radius: 125.,
                knockback: 150.,
                cooldown: 0.5,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 5.,
                speed: 600.,
                explosion_radius: 150.,
                knockback: 150.,
                cooldown: 0.5,
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_lightning_levels() -> LightningLevels {
    LightningLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 5.,
                range: 300.,
                max_hits: 3,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 10.,
                range: 300.,
                max_hits: 5,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 15.,
                range: 400.,
                max_hits: 5,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 20.,
                range: 400.,
                max_hits: 7,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 30.,
                range: 450.,
                max_hits: 10,
                cooldown: 5.,
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_scale_levels() -> ScaleLevels {
    ScaleLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 5.,
                speed: 600.,
                knockback: 1000.,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 10.,
                speed: 600.,
                knockback: 1000.,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 15.,
                speed: 600.,
                knockback: 1000.,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 20.,
                speed: 600.,
                knockback: 1000.,
                cooldown: 5.,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 30.,
                speed: 600.,
                knockback: 1000.,
                cooldown: 5.,
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_thorn_levels() -> ThornLevels {
    ThornLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 1.,
                speed: 600.,
                projectile_count: 5.,
                cooldown: 5.,
                damage_cooldown: 0.5,
                dot: DotStats {
                    damage: 1.,
                    duration: 5.,
                    tick: 1.,
                },
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 1.,
                speed: 600.,
                projectile_count: 6.,
                cooldown: 5.,
                damage_cooldown: 0.5,
                dot: DotStats {
                    damage: 2.,
                    duration: 5.,
                    tick: 1.,
                },
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 1.,
                speed: 600.,
                projectile_count: 7.,
                cooldown: 5.,
                damage_cooldown: 0.5,
                dot: DotStats {
                    damage: 2.,
                    duration: 8.,
                    tick: 1.,
                },
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 2.,
                speed: 600.,
                projectile_count: 8.,
                cooldown: 5.,
                damage_cooldown: 0.5,
                dot: DotStats {
                    damage: 3.,
                    duration: 8.,
                    tick: 1.,
                },
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 2.,
                speed: 600.,
                projectile_count: 9.,
                cooldown: 5.,
                damage_cooldown: 0.5,
                dot: DotStats {
                    damage: 4.,
                    duration: 10.,
                    tick: 1.,
                },
                ..default()
            },
        ]),
    }
}

pub(crate) fn make_orb_levels() -> OrbLevels {
    OrbLevels {
        levels: VecDeque::from([
            WeaponStats {
                level: 1.,
                damage: 5.,
                range: 75.,
                projectile_count: 3.,
                cooldown: 5.,
                lifetime: 4.,
                ..default()
            },
            WeaponStats {
                level: 2.,
                damage: 10.,
                range: 75.,
                projectile_count: 4.,
                cooldown: 5.,
                lifetime: 4.,
                ..default()
            },
            WeaponStats {
                level: 3.,
                damage: 15.,
                range: 100.,
                projectile_count: 5.,
                cooldown: 5.,
                lifetime: 4.,
                ..default()
            },
            WeaponStats {
                level: 4.,
                damage: 20.,
                range: 100.,
                projectile_count: 6.,
                cooldown: 5.,
                lifetime: 4.,
                ..default()
            },
            WeaponStats {
                level: 5.,
                damage: 30.,
                range: 150.,
                projectile_count: 7.,
                cooldown: 5.,
                lifetime: 4.,
                ..default()
            },
        ]),
    }
}
