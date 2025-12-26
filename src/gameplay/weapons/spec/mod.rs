use bevy::{ecs::system::SystemState, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    AssetStates,
    gameplay::weapons::{prelude::*, spec::loader::WeaponRonLoader},
};

pub mod apply;
pub mod components;
pub mod loader;

#[derive(AssetCollection, Resource)]
struct WeaponAssets {
    #[asset(key = "rons", collection(mapped, typed))]
    specs: HashMap<AssetFileStem, Handle<WeaponSpec>>,
}

#[derive(Resource, Deref)]
pub struct WeaponMap(HashMap<WeaponKind, WeaponSpec>);

impl FromWorld for WeaponMap {
    fn from_world(world: &mut World) -> Self {
        let mut system_state =
            SystemState::<(Res<WeaponAssets>, Res<Assets<WeaponSpec>>)>::new(world);
        let (raw_assets, spec_assets) = system_state.get(world);

        let mut map = HashMap::new();
        for (file_stem, handle) in &raw_assets.specs {
            if let Some(spec) = spec_assets.get(handle) {
                map.insert(spec.kind, spec.clone());
            } else {
                warn!("Failed to load weapon spec for: {}", file_stem.as_ref());
            }
        }

        WeaponMap(map)
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<WeaponSpec>()
        .register_asset_loader(WeaponRonLoader::new(&["weapon.ron"]));

    app.configure_loading_state(
        LoadingStateConfig::new(AssetStates::AssetLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "weapons/dynamic_weapons.ron",
            )
            .load_collection::<WeaponAssets>()
            .finally_init_resource::<WeaponMap>(),
    );
}
