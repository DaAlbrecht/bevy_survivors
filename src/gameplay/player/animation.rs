//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use rand::prelude::*;
use std::time::Duration;

use crate::{
    PausableSystems, PostPhysicsAppSystems,
    audio::SpatialPool,
    gameplay::player::{Player, PlayerAssets},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(PostPhysicsAppSystems::TickTimers),
            (
                update_animation_movement,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .in_set(PostPhysicsAppSystems::PlayAnimations),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    player: Single<(&LinearVelocity, &mut Sprite, &mut PlayerAnimation), With<Player>>,
) {
    let (linear_velocity, mut sprite, mut animation) = player.into_inner();

    let dx = linear_velocity.x;
    if dx != 0.0 {
        sprite.flip_x = dx < 0.0;
    }

    let animation_state = if **linear_velocity == Vec2::ZERO {
        PlayerAnimationState::Idling
    } else {
        PlayerAnimationState::Walking
    };
    animation.update_state(animation_state);
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&PlayerAnimation, &mut Sprite), With<Player>>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    player_assets: If<Res<PlayerAssets>>,
    mut step_query: Query<&PlayerAnimation>,
) {
    let entity = player.into_inner();

    for animation in &mut step_query {
        if animation.state == PlayerAnimationState::Walking
            && animation.changed()
            && (animation.frame == 0 || animation.frame == 3)
        {
            let rng = &mut rand::rng();
            let random_step = player_assets.steps.choose(rng).unwrap().clone();
            commands.entity(entity).with_child((
                SamplePlayer::new(random_step),
                SpatialPool,
                Transform::default(),
            ));
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
}

#[derive(Reflect, PartialEq)]
pub enum PlayerAnimationState {
    Idling,
    Walking,
}

impl PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 5;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(200);
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 5;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(120);

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            PlayerAnimationState::Walking => self.frame + 6,
            PlayerAnimationState::Idling => self.frame,
        }
    }

    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Idling,
        }
    }

    fn walking() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: PlayerAnimationState::Walking,
        }
    }

    pub fn new() -> Self {
        Self::idling()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                PlayerAnimationState::Idling => Self::IDLE_FRAMES,
                PlayerAnimationState::Walking => Self::WALKING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::Idling => *self = Self::idling(),
                PlayerAnimationState::Walking => *self = Self::walking(),
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.is_finished()
    }
}
