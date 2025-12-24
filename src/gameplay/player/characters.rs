use bevy::prelude::*;

/// The different playable Characters
#[derive(Component, Copy, Clone)]
pub(crate) enum Characters {
    Wizzard,
    Knight,
}

//TODO: Refactor into asset collections
impl Characters {
    pub(crate) const fn all() -> [Characters; 2] {
        [Characters::Wizzard, Characters::Knight]
    }

    pub(crate) fn get_spash_art(&self, asset_server: AssetServer) -> Handle<Image> {
        match self {
            Characters::Wizzard => asset_server.load("player_wizard_.png"),
            Characters::Knight => asset_server.load("player_knight_.png"),
        }
    }

    pub(crate) fn get_texture_atlas(&self) -> TextureAtlasLayout {
        match self {
            Characters::Wizzard => {
                TextureAtlasLayout::from_grid(UVec2 { x: 64, y: 64 }, 11, 1, None, None)
            }
            Characters::Knight => {
                TextureAtlasLayout::from_grid(UVec2 { x: 64, y: 64 }, 11, 1, None, None)
            }
        }
    }

    pub(crate) fn get_idle_indicies(&self) -> (usize, usize) {
        match self {
            Characters::Wizzard => (0, 5),
            Characters::Knight => (0, 4),
        }
    }
}
