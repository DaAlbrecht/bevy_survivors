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
        let mut sprite = Sprite {
            image: self.image.clone(),
            custom_size: Some(self.size),
            ..default()
        };

        if let Some(atlas) = &self.atlas {
            sprite.texture_atlas = Some(TextureAtlas {
                layout: atlas.layout.clone(),
                index: atlas.first,
            });

            ec.insert((
                AnimationIndices {
                    first: atlas.first,
                    last: atlas.last,
                },
                AnimationTimer::from_fps(atlas.fps),
            ));
        }

        ec.insert(sprite);
    }
}
