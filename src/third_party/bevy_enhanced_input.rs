use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin);
}
