use avian2d::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()));

    // The game is played on a flat plane, we do not want to have any automated Gravity.
    app.insert_resource(Gravity(Vec2::ZERO));

    #[cfg(feature = "dev")]
    app.add_plugins(PhysicsDebugPlugin);
}
