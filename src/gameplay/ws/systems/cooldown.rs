use bevy::prelude::*;

use crate::{PausableSystems, screens::Screen};

#[derive(Component, Default, Reflect)]
pub struct WeaponCooldown(pub Timer);
#[derive(Component, Reflect)]
pub struct WeaponDuration(pub Timer);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (handle_timers, tick_despawn_after)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

fn handle_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut WeaponCooldown>,
    mut durations: Query<&mut WeaponDuration>,
) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }
}

pub fn tick_despawn_after(q: Query<(Entity, &mut WeaponDuration)>, mut commands: Commands) {
    for (e, t) in q {
        if t.0.is_finished() {
            commands.entity(e).despawn();
        }
    }
}
