use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    AssetStates,
    gameplay::ws::{assets::loader::WeaponRonLoader, prelude::*},
};

#[cfg(feature = "dev")]
pub mod debug;
pub mod loader;
pub mod spec;

#[derive(AssetCollection, Resource)]
pub struct WeaponAssets {
    #[asset(key = "rons", collection(mapped, typed))]
    pub specs: HashMap<AssetFileStem, Handle<WeaponSpec>>,
}

#[allow(unused)]
impl WeaponAssets {
    pub const SPEC_SUFFIX: &'static str = ".weapon";

    #[inline]
    fn spec_key_for_id(id: &str) -> String {
        format!("{id}{}", Self::SPEC_SUFFIX)
    }

    pub fn spec_handle_for_kind(&self, kind: WeaponKind) -> Option<&Handle<WeaponSpec>> {
        let key = Self::spec_key_for_id(kind.id());
        self.specs.get(key.as_str())
    }

    pub fn spec_for_kind<'a>(
        &'a self,
        kind: WeaponKind,
        specs: &'a Assets<WeaponSpec>,
    ) -> Option<&'a WeaponSpec> {
        self.spec_handle_for_kind(kind).and_then(|h| specs.get(h))
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
            .load_collection::<WeaponAssets>(),
    );
}
