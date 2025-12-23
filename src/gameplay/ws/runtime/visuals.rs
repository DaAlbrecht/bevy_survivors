use crate::gameplay::{
    simple_animation::{AnimationIndices, AnimationTimer},
    ws::prelude::*,
};
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct WeaponProjectileVisuals(pub VisualSpec);

#[derive(Component, Clone)]
pub struct WeaponImpactVisuals(pub VisualSpec);

impl VisualSpec {
    pub fn apply_ec(&self, ec: &mut EntityCommands) {
        if let Some(atlas) = &self.atlas {
            ec.insert((
                Sprite::from_atlas_image(
                    self.image.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: atlas.first,
                    },
                ),
                AnimationIndices {
                    first: atlas.first,
                    last: atlas.last,
                },
                AnimationTimer::from_fps(atlas.fps),
            ));
        } else {
            ec.insert(Sprite {
                image: self.image.clone(),
                custom_size: Some(self.size),
                ..default()
            });
        }
    }
}
