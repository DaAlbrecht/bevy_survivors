use bevy::prelude::*;
use bevy_ecs_tiled::{
    prelude::{TiledPhysicsAvianBackend, TiledPhysicsPlugin},
    tiled::TiledPlugin,
};

pub(super) fn plugin(app: &mut App) {
    #[cfg(feature = "dev")]
    {
        use bevy_ecs_tiled::{
            prelude::{TiledFilter, regex},
            tiled::TiledPluginConfig,
        };

        let mut path = std::env::current_dir().unwrap();
        path.push("assets");
        path.push("level");
        path.push("tiled_custom_types.json");

        app.add_plugins((TiledPlugin(TiledPluginConfig {
            tiled_types_export_file: Some(path),
            tiled_types_filter: TiledFilter::from(
                regex::RegexSet::new([r"^bevy_survivors::gameplay::.*"]).unwrap(),
            ),
        }),));
    }
    #[cfg(not(feature = "dev"))]
    {
        app.add_plugins(TiledPlugin::default());
    }
    app.add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());

    // #[cfg(feature = "dev")]
    // app.add_plugins(TiledDebugPluginGroup);
}
