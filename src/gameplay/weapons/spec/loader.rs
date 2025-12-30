use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use serde::Deserialize;
use serde_ron::de::from_bytes;
use thiserror::Error;

use crate::gameplay::weapons::{
    kind::WeaponKind,
    spec::components::{AtlasAnimation, AttackSpec, HitSpec, VisualSpec, WeaponSfx, WeaponSpec},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeaponSpecRaw {
    pub kind: WeaponKind,
    pub base_damage: f32,
    pub cooldown: f32,
    pub dot: Option<f32>,
    #[serde(default)]
    pub despawn_on_hit: bool,
    pub attack: AttackSpec,
    pub on_hit: HitSpec,
    pub visuals: VisualRaw,
    pub impact_visuals: Option<VisualRaw>,
    pub sfx: SfxRaw,
    pub icon: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VisualRaw {
    pub asset_path: String,
    pub size: Vec2,
    pub atlas: Option<AtlasAnimationRaw>,
}

impl VisualRaw {
    fn load(self, load_context: &mut LoadContext<'_>) -> VisualSpec {
        let image = load_context.load(&self.asset_path);
        let atlas = self.atlas.map(|a| {
            let layout = TextureAtlasLayout::from_grid(a.cell, a.columns, a.rows, None, None);
            let label = format!(
                "{}_atlas_layout_{}",
                load_context.path().to_string_lossy(),
                self.asset_path
            );
            let layout_handle = load_context.add_labeled_asset(label, layout);

            AtlasAnimation {
                layout: layout_handle,
                first: a.first,
                last: a.last,
                fps: a.fps,
            }
        });

        VisualSpec {
            image,
            size: self.size,
            atlas,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AtlasAnimationRaw {
    pub cell: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub first: usize,
    pub last: usize,
    pub fps: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SfxRaw {
    pub attack: Option<String>,
    pub impact: Option<String>,
}

impl SfxRaw {
    fn load(self, load_context: &mut LoadContext<'_>) -> WeaponSfx {
        WeaponSfx {
            attack: self.attack.map(|p| load_context.load(p)),
            impact: self.impact.map(|p| load_context.load(p)),
        }
    }
}

//Inspired by https://github.com/NiklasEi/bevy_common_assets/blob/main/src/ron.rs
pub struct WeaponRonLoader {
    extensions: Vec<&'static str>,
}

impl WeaponRonLoader {
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum WeaponRonLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse RON: {0}")]
    RonError(#[from] serde_ron::error::SpannedError),
}

impl AssetLoader for WeaponRonLoader {
    type Asset = WeaponSpec;
    type Settings = ();
    type Error = WeaponRonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let raw = from_bytes::<WeaponSpecRaw>(&bytes)?;

        Ok(WeaponSpec {
            kind: raw.kind,
            base_damage: raw.base_damage,
            cooldown: raw.cooldown,
            dot: raw.dot,
            despawn_on_hit: raw.despawn_on_hit,
            attack: raw.attack,
            on_hit: raw.on_hit,
            visuals: raw.visuals.load(load_context),
            impact_visuals: raw.impact_visuals.map(|v| v.load(load_context)),
            sfx: raw.sfx.load(load_context),
            icon: load_context.load(raw.icon),
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn weapon_ron_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
        fn is_weapon_ron(path: &Path) -> bool {
            matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some(name) if name.ends_with(".weapon.ron")
            )
        }

        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if is_weapon_ron(&path) {
                    out.push(path);
                }
            }
        }

        let mut out = Vec::new();
        walk(root.as_ref(), &mut out);
        out.sort();
        out
    }

    fn parse_weapon_ron(path: &Path) -> Result<WeaponSpecRaw, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("{}: read error: {e}", path.display()))?;

        serde_ron::de::from_bytes::<WeaponSpecRaw>(&bytes)
            .map_err(|e| format!("{}: parse error: {e}", path.display()))
    }

    fn require_files_exist(
        base: &Path,
        origin: &Path,
        rels: &[(&'static str, &str)],
    ) -> Vec<String> {
        rels.iter()
            .filter_map(|(label, rel)| {
                let p = base.join(rel);
                (!p.exists()).then(|| {
                    format!(
                        "{}: {label} missing: {rel} (looked for {})",
                        origin.display(),
                        p.display()
                    )
                })
            })
            .collect()
    }

    #[test]
    fn weapon_ron_files_parse() {
        let files = weapon_ron_files("assets/weapons/rons");
        assert!(!files.is_empty(), "No .weapon.ron files found");

        let failures: Vec<String> = files
            .iter()
            .filter_map(|p| parse_weapon_ron(p).err())
            .collect();

        assert!(
            failures.is_empty(),
            "Weapon RON validation failed for {} file(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn weapon_ron_references_existing_files() {
        let files = weapon_ron_files("assets/weapons/rons");
        assert!(!files.is_empty(), "No .weapon.ron files found");

        let asset_root = Path::new("assets");

        let failures: Vec<String> = files
            .iter()
            .flat_map(|p| match parse_weapon_ron(p) {
                Ok(raw) => {
                    let mut refs = vec![("visuals.image", raw.visuals.asset_path.as_str())];
                    if let Some(s) = raw.impact_visuals.as_ref().map(|v| v.asset_path.as_str()) {
                        refs.push(("impact_visuals.image", s));
                    }
                    refs.push(("icon", raw.icon.as_str()));
                    if let Some(s) = raw.sfx.attack.as_deref() {
                        refs.push(("sfx.attack", s));
                    }
                    if let Some(s) = raw.sfx.impact.as_deref() {
                        refs.push(("sfx.impact", s));
                    }
                    require_files_exist(asset_root, p, &refs)
                }
                Err(e) => vec![e],
            })
            .collect();

        assert!(
            failures.is_empty(),
            "Weapon reference validation failed for {} file(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn every_weaponkind_has_a_weapon_ron() {
        use std::collections::HashSet;

        let files = weapon_ron_files("assets/weapons/rons");
        assert!(!files.is_empty(), "No .weapon.ron files found");

        let mut seen = HashSet::new();
        let mut failures = Vec::new();

        for p in &files {
            match parse_weapon_ron(p) {
                Ok(raw) => {
                    seen.insert(raw.kind);
                }
                Err(e) => failures.push(e),
            }
        }

        assert!(
            failures.is_empty(),
            "Weapon kind scan failed for {} file(s):\n{}",
            failures.len(),
            failures.join("\n")
        );

        let missing: Vec<WeaponKind> = WeaponKind::ALL
            .iter()
            .copied()
            .filter(|k| !seen.contains(k))
            .collect();

        assert!(
            missing.is_empty(),
            "No .weapon.ron found for WeaponKind variant(s): {missing:?}",
        );
    }
}
