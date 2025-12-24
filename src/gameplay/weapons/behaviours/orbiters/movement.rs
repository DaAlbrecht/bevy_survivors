use crate::gameplay::player::Player;
use bevy::prelude::*;

use super::{OrbitAngularSpeed, OrbitPhase, OrbitRadius, OrbiterProjectile};

use crate::{PausableSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_orbiters)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

pub fn move_orbiters(
    player_q: Query<&Transform, (With<Player>, Without<OrbiterProjectile>)>,
    mut orb_q: Query<
        (
            &mut Transform,
            &mut OrbitPhase,
            &OrbitRadius,
            &OrbitAngularSpeed,
        ),
        (With<OrbiterProjectile>, Without<Player>),
    >,
    time: Res<Time<Fixed>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let dt = time.delta_secs();

    for (mut tf, mut phase, radius, ang_speed) in &mut orb_q {
        phase.0 = (phase.0 + ang_speed.0 * dt) % std::f32::consts::TAU;
        let offset = Vec2::from_angle(phase.0) * radius.0;
        tf.translation = player_tf.translation + offset.extend(tf.translation.z);
    }
}
