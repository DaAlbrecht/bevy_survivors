use bevy::prelude::{App, PostUpdate};
use bevy::{ecs::schedule::IntoScheduleConfigs, render::camera};

use crate::camera::zoom::pixel_zoom_system;

pub mod zoom;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, pixel_zoom_system.after(camera::camera_system));
}
