use crate::gameplay::player::Player;
use bevy::prelude::*;

use crate::{PausableSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_zone_attack)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

pub fn move_zone_attack(
    player_q: Query<&Transform, (With<Player>, Without<super::ZoneFollowPlayer>)>,
    mut zone_q: Query<&mut Transform, (With<super::ZoneFollowPlayer>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    for mut tf in &mut zone_q {
        tf.translation = player_tf.translation;
    }
}
