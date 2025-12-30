use bevy::prelude::*;

use crate::gameplay::items::loader::{ItemRegistry, ModRule};
use crate::gameplay::items::stats::{ItemModifiers, RecalculateStats, Stats};
use crate::gameplay::player::InInventoryOf;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    Weapon,
    Passive,
}

/// Rarity is fixed per item. E.g. some are more rare than others.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    pub fn color(&self) -> Color {
        match self {
            ItemRarity::Common => Color::srgb(0.7, 0.7, 0.7),
            ItemRarity::Uncommon => Color::srgb(0.3, 1.0, 0.3),
            ItemRarity::Rare => Color::srgb(0.3, 0.5, 1.0),
            ItemRarity::Epic => Color::srgb(0.7, 0.3, 1.0),
            ItemRarity::Legendary => Color::srgb(1.0, 0.6, 0.0),
        }
    }
}
#[derive(Component, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ItemId(pub String);

#[derive(Component, Copy, Clone, Debug)]
pub struct ItemLevel(pub u32);

#[derive(EntityEvent)]
pub struct UpgradeItem {
    pub entity: Entity, // target item
    pub amount: i32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_upgrade_item);

    app.add_systems(Update, recompute_item_modifiers_from_registry);
}

fn on_upgrade_item(
    ev: On<UpgradeItem>,
    mut q: Query<(&mut ItemLevel, &InInventoryOf)>,
    mut commands: Commands,
) {
    if let Ok((mut lvl, equipped_to)) = q.get_mut(ev.entity) {
        lvl.0 = (lvl.0 as i32 + ev.amount).max(0) as u32;
        // Explicitly trigger recalculation for the owner
        info!(
            "Item upgraded to level {}, triggering recalc for owner {:?}",
            lvl.0, equipped_to.0
        );
        commands.trigger(RecalculateStats {
            entity: equipped_to.0,
        });
    }
}

fn recompute_item_modifiers_from_registry(
    reg: Res<ItemRegistry>,
    mut q: Query<
        (&ItemId, &ItemLevel, &mut ItemModifiers),
        Or<(Changed<ItemId>, Changed<ItemLevel>, Added<ItemModifiers>)>,
    >,
) {
    for (id, level, mut mods) in &mut q {
        let Some(def) = reg.get(&id.0) else {
            *mods = ItemModifiers::default();
            continue;
        };

        // Reset to identity each recompute.
        mods.add = Stats::zero();
        mods.mul = Stats::one();

        for entry in def.rules.iter().copied() {
            let value = entry.rule.value_at(level.0);
            match entry.rule {
                ModRule::LinearAdd { .. } => {
                    if value != 0.0 {
                        *mods.add.get_mut(entry.stat) += value;
                    }
                }
                ModRule::ExpMul { .. } => {
                    if value != 1.0 {
                        *mods.mul.get_mut(entry.stat) *= value;
                    }
                }
            }
        }
    }
}
