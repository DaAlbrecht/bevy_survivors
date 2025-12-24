use bevy::prelude::*;
use bevy::state::condition::in_state;

use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (animate_ui_sprites,).run_if(in_state(Screen::Title)),
    );
}

fn animate_ui_sprites(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut ImageNode)>,
) {
    for (indices, mut timer, mut image_node) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut image_node.texture_atlas
        {
            if atlas.index == indices.last {
                atlas.index = indices.first;
            } else {
                atlas.index += 1;
            };
        }
    }
}
