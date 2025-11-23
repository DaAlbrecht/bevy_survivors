use bevy::{platform::collections::HashMap, prelude::*};
use std::collections::VecDeque;

use crate::gameplay::enemy::EnemyType;

use super::{WavePlan, WaveStats};

//Later read json or someting
pub(crate) fn make_wave_plan() -> WavePlan {
    WavePlan {
        waves: VecDeque::from([
            WaveStats {
                enemy_pool: HashMap::from([(EnemyType::Walker, 0.7), (EnemyType::Shooter, 0.3)]),
                enemy_screen_count: 30.0,
                spawn_frequency: 1.0,
                duration: 60.0,
                power_level: 1.0, //Does not change Basestats
                sprite_pool: HashMap::from([
                    (EnemyType::Walker, "enemies/walker.png".to_string()),
                    (EnemyType::Shooter, "enemies/shooter.png".to_string()),
                ]),
            },
            WaveStats {
                enemy_pool: HashMap::from([
                    (EnemyType::Walker, 0.5),
                    (EnemyType::Shooter, 0.3),
                    (EnemyType::Sprinter, 0.2),
                ]),
                enemy_screen_count: 20.0,
                spawn_frequency: 1.5,
                duration: 60.0,
                power_level: 2.0,
                sprite_pool: HashMap::from([
                    (EnemyType::Walker, "enemies/walker_blue.png".to_string()),
                    (
                        EnemyType::Sprinter,
                        "enemies/sprinter_purple.png".to_string(),
                    ),
                    (EnemyType::Shooter, "enemies/shooter_yellow.png".to_string()),
                ]),
            },
            WaveStats {
                enemy_pool: HashMap::from([
                    (EnemyType::Walker, 0.6),
                    (EnemyType::Sprinter, 0.2),
                    (EnemyType::Shooter, 0.2),
                ]),
                enemy_screen_count: 30.0,
                spawn_frequency: 2.0,
                duration: 60.0,
                power_level: 2.0,
                sprite_pool: HashMap::from([
                    (EnemyType::Walker, "enemies/walker_yellow.png".to_string()),
                    (
                        EnemyType::Sprinter,
                        "enemies/sprinter_purple.png".to_string(),
                    ),
                    (EnemyType::Shooter, "enemies/shooter_purple.png".to_string()),
                ]),
            },
        ]),
    }
}
