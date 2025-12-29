use crate::gameplay::{player::Player, weapons::behaviours::melee::MeleeAttackZone};
use bevy::prelude::*;

use crate::{PausableSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_melee_zones)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

pub fn move_melee_zones(
    player_q: Query<&Transform, (With<Player>, Without<MeleeAttackZone>)>,
    mut zone_q: Query<&mut Transform, (With<MeleeAttackZone>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    for mut tf in &mut zone_q {
        tf.translation = player_tf.translation;
    }
}
