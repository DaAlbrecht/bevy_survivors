use std::time::Duration;

use bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, animate_sprite.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component)]
pub(crate) struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AnimationTimer {
    pub timer: Timer,
}

impl AnimationTimer {
    pub fn once_from_fps(fps: u8) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once),
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (entity, indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index == indices.last {
                atlas.index = indices.first;
                if timer.mode() == TimerMode::Once {
                    info!("Despawning animated entity {:?}", entity);
                    commands.entity(entity).despawn();
                }
            } else {
                atlas.index += 1;
                if timer.mode() == TimerMode::Once {
                    timer.reset();
                }
            };
        }
    }
}
