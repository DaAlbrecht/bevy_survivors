use std::collections::HashMap;

use bevy::prelude::*;

use crate::gameplay::items::{
    items::{ItemCategory, ItemRarity},
    stats::{StatId, Stats},
};

#[derive(Clone, Copy, Debug)]
pub enum ModRule {
    /// add = base + per_level * level
    LinearAdd { base: f32, per_level: f32 },
    /// mul = base * (per_level ^ level)  (1.0-based)
    ExpMul { base: f32, per_level: f32 },
}

impl ModRule {
    pub(crate) fn value_at(&self, level: u32) -> f32 {
        let level = level as f32;
        match *self {
            ModRule::LinearAdd { base, per_level } => base + per_level * level,
            ModRule::ExpMul { base, per_level } => base * per_level.powf(level),
        }
    }

    pub(crate) fn delta_between(&self, from: u32, to: u32) -> f32 {
        self.value_at(to) - self.value_at(from)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RuleEntry {
    pub stat: StatId,
    pub rule: ModRule,
}

#[derive(Clone, Debug)]
pub struct ItemSpec {
    pub name: &'static str,
    pub category: ItemCategory,
    pub rarity: ItemRarity,
    pub max_level: u32,
    pub evolution: Option<String>,
    pub evolution_requirement: Option<String>,
    pub rules: Vec<RuleEntry>,
}

impl ItemSpec {
    fn for_each_value<F>(&self, level: u32, mut f: F)
    where
        F: FnMut(StatId, f32),
    {
        for entry in &self.rules {
            f(entry.stat, entry.rule.value_at(level));
        }
    }

    fn for_each_delta<F>(&self, from: u32, to: u32, mut f: F)
    where
        F: FnMut(StatId, f32),
    {
        for entry in &self.rules {
            f(entry.stat, entry.rule.delta_between(from, to));
        }
    }

    pub fn new(name: &'static str, category: ItemCategory) -> Self {
        Self {
            name,
            category,
            rarity: ItemRarity::Common,
            max_level: 8,
            evolution: None,
            evolution_requirement: None,
            rules: Vec::new(),
        }
    }

    pub fn with_rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = rarity;
        self
    }

    pub fn with_max_level(mut self, max_level: u32) -> Self {
        self.max_level = max_level;
        self
    }

    pub fn with_evolution(
        mut self,
        evolves_into: impl Into<String>,
        requires: impl Into<String>,
    ) -> Self {
        self.evolution = Some(evolves_into.into());
        self.evolution_requirement = Some(requires.into());
        self
    }

    pub fn with_rule(mut self, stat: StatId, rule: ModRule) -> Self {
        self.rules.push(RuleEntry { stat, rule });
        self
    }

    pub fn linear_add(self, stat: StatId, base: f32, per_level: f32) -> Self {
        self.with_rule(stat, ModRule::LinearAdd { base, per_level })
    }

    pub fn exp_mul(self, stat: StatId, base: f32, per_level: f32) -> Self {
        self.with_rule(stat, ModRule::ExpMul { base, per_level })
    }

    /// Get color from rarity
    pub fn color(&self) -> Color {
        self.rarity.color()
    }

    /// Get calculated stats at given level as `Stats`
    pub fn calculate_stats(&self, level: u32) -> Stats {
        let mut stats = Stats::zero();
        self.for_each_value(level, |stat, value| {
            stats.set(stat, value);
        });
        stats
    }

    /// Get formatted stat changes for this item at a specific level
    /// Only shows stats that this item actually modifies
    /// Used for ui
    pub fn format_stats_at_level(&self, level: u32) -> String {
        let mut text = String::new();

        self.for_each_value(level, |stat, value| {
            if value.abs() > 0.001 {
                let formatted = stat.format_value(value);
                text.push_str(&format!("{}: {}\n", stat.display_name(), formatted));
            }
        });

        text
    }

    /// Get formatted upgrade text showing what changes from current to next level
    /// Only shows stats that this item actually modifies
    pub fn format_upgrade(&self, current_level: u32, next_level: u32) -> String {
        let mut text = String::new();

        self.for_each_delta(current_level, next_level, |stat, delta| {
            if delta.abs() > 0.001 {
                let formatted = stat.format_delta(delta);
                text.push_str(&format!("{}: {}\n", stat.display_name(), formatted));
            }
        });

        text
    }
}

#[derive(Resource, Default)]
pub struct ItemRegistry {
    defs: HashMap<String, ItemSpec>,
}

impl ItemRegistry {
    pub fn insert(&mut self, id: impl Into<String>, def: ItemSpec) {
        self.defs.insert(id.into(), def);
    }

    pub fn get(&self, id: &str) -> Option<&ItemSpec> {
        self.defs.get(id)
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ItemRegistry>();

    app.add_systems(Startup, register_builtin_items);
}

fn register_builtin_items(mut reg: ResMut<ItemRegistry>) {
    // Passive items
    reg.insert(
        "tome_crit",
        ItemSpec::new("Crit Tome", ItemCategory::Passive)
            .with_rarity(ItemRarity::Uncommon)
            .linear_add(StatId::CritChance, 0.0, 0.02),
    );

    reg.insert(
        "tome_attack_speed",
        ItemSpec::new("Attack Speed Tome", ItemCategory::Passive)
            .with_rarity(ItemRarity::Rare)
            .exp_mul(StatId::AttackSpeed, 1.0, 1.05),
    );

    reg.insert(
        "health",
        ItemSpec::new("Health", ItemCategory::Passive)
            .with_rarity(ItemRarity::Common)
            .linear_add(StatId::Attack, 0.0, 10.0),
    );

    reg.insert(
        "wings",
        ItemSpec::new("Wings", ItemCategory::Passive)
            .with_rarity(ItemRarity::Common)
            .exp_mul(StatId::MoveSpeed, 1.0, 1.1),
    );
}
