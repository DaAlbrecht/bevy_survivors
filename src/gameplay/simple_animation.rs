use std::time::Duration;

use bevy::prelude::*;

use crate::{gameplay::enemy::Root, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (animate_sprite, hurt_flash, root_flash).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Clone, Copy, Default)]
pub enum AnimationPlayback {
    #[default]
    Loop,
    OnceDespawn,
}

#[derive(Component, Clone)]
pub(crate) struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut, Clone)]
#[require(AnimationPlayback)]
pub(crate) struct AnimationTimer {
    pub timer: Timer,
}

impl AnimationTimer {
    pub fn from_fps(fps: u8) -> Self {
        Self {
            timer: Timer::new(
                Duration::from_secs_f32(1.0 / (fps as f32)),
                TimerMode::Repeating,
            ),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct HurtAnimationTimer {
    pub timer: Timer,
}

impl Default for HurtAnimationTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(1.0 / 6.), TimerMode::Once),
        }
    }
}

fn hurt_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut HurtAnimationTimer, &mut Sprite)>,
) {
    for (entity, mut hurt_timer, mut sprite) in &mut query {
        hurt_timer.tick(time.delta());

        if hurt_timer.just_finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<HurtAnimationTimer>();
        } else {
            sprite.color = Color::srgba(1.0, 0.0, 0.0, 1.0);
        }
    }
}

fn root_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Root, &mut Sprite)>,
) {
    for (entity, mut root, mut sprite) in &mut query {
        root.0.tick(time.delta());

        if root.0.just_finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<Root>();
        } else {
            sprite.color = Color::srgba(0.5, 1.0, 1.0, 1.0);
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &AnimationIndices,
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationPlayback,
    )>,
) {
    for (entity, indices, mut timer, mut sprite, playback) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index >= indices.last {
                match *playback {
                    AnimationPlayback::Loop => atlas.index = indices.first,
                    AnimationPlayback::OnceDespawn => commands.entity(entity).despawn(),
                }
            } else {
                atlas.index += 1;
            }
        }
    }
}
