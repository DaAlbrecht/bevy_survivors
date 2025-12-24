use crate::gameplay::weapons::prelude::*;
use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;

/// Debugging system that logs the status of loaded weapon assets.
/// Wired to F5
pub fn debug_weapon_assets(
    weapon_assets: Res<WeaponMap>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    audio_samples: Res<Assets<AudioSample>>,
) {
    info!("================ WeaponMap DEBUG ================");

    info!("--- loaded spec keys ---");
    for k in weapon_assets.keys() {
        info!("spec key: {:?}", k);
    }

    let kinds = [WeaponKind::Orb, WeaponKind::Lightning];

    for kind in kinds {
        let Some(spec) = weapon_assets.get(&kind) else {
            error!("Missing spec for kind {:?}", kind);
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
            "Weapon {:?}: cooldown={}, image_ok={}, atlas_ok={}, attack_sfx_ok={}, impact_sfx_ok={}",
            kind, spec.cooldown, image_ok, atlas_ok, attack_sfx_ok, impact_sfx_ok,
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
