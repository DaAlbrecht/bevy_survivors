use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;
use serde::Deserialize;
use serde_ron::de::from_bytes;
use thiserror::Error;

use crate::gameplay::ws::prelude::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeaponSpecRaw {
    pub name: String,
    pub base_damage: f32,
    pub cooldown: f32,
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
    pub image: String,
    pub size: Vec2,
    pub z: f32,
    pub atlas: Option<AtlasAnimRaw>,
}

impl VisualRaw {
    fn into_visual_spec(self, load_context: &mut LoadContext<'_>) -> VisualSpec {
        let image: Handle<Image> = load_context.load(self.image);
        let atlas = self.atlas.map(|a| {
            let layout = TextureAtlasLayout::from_grid(a.cell, a.columns, a.rows, None, None);

            let label = format!("{}_atlas_layout", load_context.path().to_string_lossy());
            let layout_handle = load_context.add_labeled_asset(label, layout);

            AtlasAnim {
                layout: layout_handle,
                first: a.first,
                last: a.last,
                fps: a.fps,
            }
        });

        VisualSpec {
            image,
            size: self.size,
            z: self.z,
            atlas,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AtlasAnimRaw {
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
        let WeaponSpecRaw {
            name,
            base_damage,
            cooldown,
            attack,
            on_hit,
            visuals,
            impact_visuals,
            sfx,
            icon,
        } = raw;

        let attack_sfx = sfx.attack.map(|p| load_context.load::<AudioSample>(p));

        let impact_sfx = sfx.impact.map(|p| load_context.load::<AudioSample>(p));

        let icon_handle: Handle<Image> = load_context.load(icon);

        Ok(WeaponSpec {
            name,
            base_damage,
            cooldown,
            attack,
            on_hit,
            visuals: visuals.into_visual_spec(load_context),
            impact_visuals: impact_visuals.map(|i| i.into_visual_spec(load_context)),
            sfx: WeaponSfx {
                attack: attack_sfx,
                impact: impact_sfx,
            },
            icon: icon_handle,
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
            let entries = match std::fs::read_dir(dir) {
                Ok(rd) => rd,
                Err(_) => return,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                match path.is_dir() {
                    true => walk(&path, out),
                    false if is_weapon_ron(&path) => out.push(path),
                    _ => {}
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

        match failures.is_empty() {
            true => {}
            false => panic!(
                "Weapon RON validation failed for {} file(s):\n{}",
                failures.len(),
                failures.join("\n")
            ),
        }
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
                    let mut refs = vec![("visuals.image", raw.visuals.image.as_str())];
                    if let Some(s) = raw.impact_visuals.as_ref().map(|v| v.image.as_str()) {
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

        match failures.is_empty() {
            true => {}
            false => panic!(
                "Weapon reference validation failed for {} file(s):\n{}",
                failures.len(),
                failures.join("\n")
            ),
        }
    }
}
