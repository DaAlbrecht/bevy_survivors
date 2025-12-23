use bevy::prelude::*;

/// The different playable Characters
#[derive(Component)]
pub(crate) enum Characters {
    Wizzard,
    Knight,
}

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
}
