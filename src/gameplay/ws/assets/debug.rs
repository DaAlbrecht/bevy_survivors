use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;

/// Debugging system that logs the status of loaded weapon assets.
/// Wired to F5
pub fn debug_weapon_assets(
    weapon_assets: Res<WeaponAssets>,
    weapon_specs: Res<Assets<WeaponSpec>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    audio_samples: Res<Assets<AudioSample>>,
) {
    info!("================ WeaponAssets DEBUG ================");

    info!("--- loaded spec keys ---");
    for k in weapon_assets.specs.keys() {
        info!("spec key: {:?}", k);
    }

    let kinds = [WeaponKind::Orb, WeaponKind::Lightning];

    for kind in kinds {
        let stem = format!("{}.weapon", kind.id());
        let Some(spec_handle) = weapon_assets.specs.get(stem.as_str()) else {
            error!(
                "Missing spec handle for kind {:?} (expected key '{}')",
                kind, stem
            );
            continue;
        };

        let Some(spec) = weapon_specs.get(spec_handle) else {
            error!(
                "Spec asset not resolved for kind {:?} (handle id={:?})",
                kind,
                spec_handle.id()
            );
            continue;
        };

        let image_ok = images.get(&spec.visuals.image).is_some();
        let atlas_ok = spec
            .visuals
            .atlas
            .as_ref()
            .map(|a| atlas_layouts.get(&a.layout).is_some())
            .unwrap_or(true);

        let attack_sfx_ok = spec
            .sfx
            .attack
            .as_ref()
            .map(|h| audio_samples.get(h).is_some())
            .unwrap_or(true);

        let impact_sfx_ok = spec
            .sfx
            .impact
            .as_ref()
            .map(|h| audio_samples.get(h).is_some())
            .unwrap_or(true);

        info!(
            "Weapon {:?}: name='{}', cooldown={}, image_ok={}, atlas_ok={}, attack_sfx_ok={}, impact_sfx_ok={}",
            kind, spec.name, spec.cooldown, image_ok, atlas_ok, attack_sfx_ok, impact_sfx_ok,
        );

        if !image_ok {
            error!("  -> visuals.image handle not resolved for {:?}", kind);
        }
        if !atlas_ok {
            error!(
                "  -> visuals.atlas.layout handle not resolved for {:?}",
                kind
            );
        }
        if !attack_sfx_ok {
            error!("  -> sfx.attack handle not resolved for {:?}", kind);
        }
        if !impact_sfx_ok {
            error!("  -> sfx.impact handle not resolved for {:?}", kind);
        }
    }

    info!("====================================================");
}
