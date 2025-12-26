use crate::gameplay::{
    simple_animation::{AnimationIndices, AnimationTimer},
    weapons::prelude::*,
};
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct WeaponProjectileVisuals(pub VisualSpec);

#[derive(Component, Clone)]
pub struct WeaponImpactVisuals(pub VisualSpec);

impl VisualSpec {
    pub fn apply_ec(&self, ec: &mut EntityCommands) {
        ec.insert(self.get_sprite());

        if let Some(atlas) = &self.atlas {
            ec.insert((
                AnimationIndices {
                    first: atlas.first,
                    last: atlas.last,
                },
                AnimationTimer::from_fps(atlas.fps),
            ));
        }
    }

    pub fn get_sprite(&self) -> Sprite {
        if let Some(atlas) = &self.atlas {
            Sprite::from_atlas_image(
                self.image.clone(),
                TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: atlas.first,
                },
            )
        } else {
            Sprite {
                image: self.image.clone(),
                custom_size: Some(self.size),
                ..default()
            }
        }
    }

    /// Duration (seconds) to play the atlas exactly once at its fps.
    /// Returns `fallback_secs` if there is no atlas.
    pub fn duration_secs_once_or(&self, fallback_secs: f32) -> f32 {
        let Some(atlas) = &self.atlas else {
            return fallback_secs;
        };

        let fps_frames = atlas.fps.max(1) as f32;

        let frames = (atlas.last - atlas.first + 1) as f32;

        frames / fps_frames
    }
}
