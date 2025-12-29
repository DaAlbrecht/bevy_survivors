use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::HashMap;

use crate::gameplay::player::Inventory;

use super::items::{ItemId, ItemLevel};

/// SysParam player inventory items relations
#[derive(SystemParam)]
pub struct PlayerInventory<'w, 's> {
    inventory: Query<'w, 's, &'static Inventory>,
    items: Query<'w, 's, (&'static ItemId, &'static ItemLevel)>,
}

impl<'w, 's> PlayerInventory<'w, 's> {
    /// Get all items owned by the given entity (player)
    pub fn get_items(&self, owner: Entity) -> impl Iterator<Item = (Entity, &ItemId, &ItemLevel)> {
        self.inventory
            .iter_descendants(owner)
            .filter_map(|item_entity| {
                self.items
                    .get(item_entity)
                    .ok()
                    .map(|(id, level)| (item_entity, id, level))
            })
    }

    /// Get a specific item by ID
    pub fn get_item(&self, owner: Entity, item_id: &str) -> Option<(Entity, &ItemLevel)> {
        self.get_items(owner)
            .find(|(_, id, _)| id.0 == item_id)
            .map(|(entity, _, level)| (entity, level))
    }

    /// Get the current level of an item
    pub fn get_item_level(&self, owner: Entity, item_id: &str) -> Option<u32> {
        self.get_item(owner, item_id).map(|(_, level)| level.0)
    }

    /// Build a HashMap of item IDs to their current levels for the given owner
    pub fn build_level_map(&self, owner: Entity) -> HashMap<String, u32> {
        self.get_items(owner)
            .map(|(_, id, level)| (id.0.clone(), level.0))
            .collect()
    }

    /// Check if the owner has a specific item
    pub fn has_item(&self, owner: Entity, item_id: &str) -> bool {
        self.get_item(owner, item_id).is_some()
    }
}
