use bevy::prelude::*;
use std::ops::{Add, AddAssign, Mul, MulAssign};

use crate::gameplay::player::{InInventoryOf, Inventory};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum StatId {
    // Damage stats
    Attack,
    CritChance,
    CritDamage,
    AttackSpeed,

    // Movement & survivability
    MoveSpeed,
    MaxHealth,
    Armor,
    Recovery,

    // Weapon modifiers
    ProjectileCount,
    Duration,
    Area,
    Cooldown,

    // Utility
    PickupRange,
    // Luck,
    // Greed,
}

impl StatId {
    pub const COUNT: usize = 16;

    /// Get the display name for this stat
    pub fn display_name(self) -> &'static str {
        match self {
            StatId::Attack => "Attack",
            StatId::CritChance => "Crit Chance",
            StatId::CritDamage => "Crit Damage",
            StatId::AttackSpeed => "Attack Speed",
            StatId::MoveSpeed => "Move Speed",
            StatId::MaxHealth => "Max Health",
            StatId::Armor => "Armor",
            StatId::Recovery => "Recovery",
            StatId::ProjectileCount => "Projectile Count",
            StatId::Duration => "Duration",
            StatId::Area => "Area",
            StatId::Cooldown => "Cooldown",
            StatId::PickupRange => "Pickup Range",
        }
    }

    /// Format a stat value for display
    pub fn format_value(self, value: f32) -> String {
        match self {
            StatId::CritChance => format!("{:.1}%", value * 100.0),
            StatId::CritDamage => format!("{:.0}%", (value - 1.0) * 100.0),
            StatId::Armor => format!("{:.1}%", value * 100.0),
            StatId::AttackSpeed
            | StatId::MoveSpeed
            | StatId::Duration
            | StatId::Area
            | StatId::Cooldown => {
                format!("{:.0}%", (value - 1.0) * 100.0)
            }
            _ => format!("{:.1}", value),
        }
    }

    /// Format a stat delta/change for display (e.g., for level-up screen)
    pub fn format_delta(self, delta: f32) -> String {
        match self {
            StatId::CritChance => {
                let percent = delta * 100.0;
                if percent >= 0.0 {
                    format!("+{:.1}%", percent)
                } else {
                    format!("{:.1}%", percent)
                }
            }
            StatId::CritDamage => {
                let percent = delta * 100.0;
                if percent >= 0.0 {
                    format!("+{:.0}%", percent)
                } else {
                    format!("{:.0}%", percent)
                }
            }
            StatId::Armor => {
                let percent = delta * 100.0;
                if percent >= 0.0 {
                    format!("+{:.1}%", percent)
                } else {
                    format!("{:.1}%", percent)
                }
            }
            StatId::AttackSpeed
            | StatId::MoveSpeed
            | StatId::Duration
            | StatId::Area
            | StatId::Cooldown => {
                let percent = delta * 100.0;
                if percent >= 0.0 {
                    format!("+{:.0}%", percent)
                } else {
                    format!("{:.0}%", percent)
                }
            }
            _ => {
                if delta >= 0.0 {
                    format!("+{:.1}", delta)
                } else {
                    format!("{:.1}", delta)
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    values: [f32; StatId::COUNT],
}

impl Default for Stats {
    fn default() -> Self {
        Self::zero()
    }
}

impl Stats {
    #[inline]
    pub fn zero() -> Self {
        Self {
            values: [0.0; StatId::COUNT],
        }
    }

    /// used for multiplication
    #[inline]
    pub fn one() -> Self {
        Self {
            values: [1.0; StatId::COUNT],
        }
    }

    #[inline]
    pub fn get(&self, id: StatId) -> f32 {
        self.values[id as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, id: StatId) -> &mut f32 {
        &mut self.values[id as usize]
    }

    #[inline]
    pub fn set(&mut self, id: StatId, value: f32) {
        self.values[id as usize] = value;
    }

    #[inline]
    pub fn add_stat(&mut self, id: StatId, amount: f32) {
        self.values[id as usize] += amount;
    }

    #[inline]
    pub fn clamp_rules(mut self) -> Self {
        *self.get_mut(StatId::CritChance) = self.get(StatId::CritChance).clamp(0.0, 1.0);
        *self.get_mut(StatId::AttackSpeed) = self.get(StatId::AttackSpeed).max(0.05);
        *self.get_mut(StatId::MoveSpeed) = self.get(StatId::MoveSpeed).max(0.1);
        *self.get_mut(StatId::MaxHealth) = self.get(StatId::MaxHealth).max(1.0);
        *self.get_mut(StatId::Armor) = self.get(StatId::Armor).clamp(0.0, 0.99);
        *self.get_mut(StatId::Cooldown) = self.get(StatId::Cooldown).max(0.1);
        self
    }

    /// Format a stat value for display
    #[inline]
    pub fn format(self, stat_id: StatId) -> String {
        stat_id.format_value(self.get(stat_id))
    }
}

impl Add for Stats {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        for i in 0..StatId::COUNT {
            result.values[i] = self.values[i] + rhs.values[i];
        }
        result
    }
}

impl AddAssign for Stats {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..StatId::COUNT {
            self.values[i] += rhs.values[i];
        }
    }
}

impl Mul for Stats {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        for i in 0..StatId::COUNT {
            result.values[i] = self.values[i] * rhs.values[i];
        }
        result
    }
}

impl MulAssign for Stats {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..StatId::COUNT {
            self.values[i] *= rhs.values[i];
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BaseStats(pub Stats);

impl Default for BaseStats {
    fn default() -> Self {
        let mut stats = Stats::zero();

        // Set multiplier-based stats to 1.0 (100% baseline)
        stats.set(StatId::CritDamage, 1.5); // 150% crit damage baseline
        stats.set(StatId::AttackSpeed, 1.0);
        stats.set(StatId::MoveSpeed, 1.0);
        stats.set(StatId::Duration, 1.0);
        stats.set(StatId::Area, 1.0);
        stats.set(StatId::Cooldown, 1.0);

        // Set additive stats with reasonable defaults
        stats.set(StatId::Attack, 10.0);
        stats.set(StatId::MaxHealth, 100.0);
        stats.set(StatId::PickupRange, 50.0);
        stats.set(StatId::ProjectileCount, 1.0);

        Self(stats)
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct UpgradeStats(pub Stats);

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct DerivedStats(pub Stats);

#[derive(Component, Debug, Clone, Copy)]
pub struct ItemModifiers {
    /// Flat additions (e.g. +10 attack)
    pub add: Stats,
    /// Multipliers (1.0 = no change). e.g. 1.10 for +10%
    pub mul: Stats,
}

impl Default for ItemModifiers {
    fn default() -> Self {
        Self {
            add: Stats::zero(),
            mul: Stats::one(),
        }
    }
}

#[derive(EntityEvent)]
pub struct AddUpgrade {
    pub entity: Entity,
    pub stat: StatId,
    pub amount: f32,
}

#[derive(EntityEvent)]
pub struct RecalculateStats {
    pub entity: Entity,
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_add_upgrade);
    app.add_observer(on_item_modifiers_insert);
    app.add_observer(on_item_modifiers_replace);
    app.add_observer(on_recalculate_stats);
}

fn on_add_upgrade(ev: On<AddUpgrade>, mut commands: Commands, mut q: Query<&mut UpgradeStats>) {
    if let Ok(mut upgrades) = q.get_mut(ev.entity) {
        upgrades.0.add_stat(ev.stat, ev.amount);
        commands.trigger(RecalculateStats { entity: ev.entity });
    }
}

fn on_item_modifiers_insert(
    ev: On<Insert, ItemModifiers>,
    mut commands: Commands,
    q_equipped_to: Query<&InInventoryOf>,
) {
    if let Ok(equipped_to) = q_equipped_to.get(ev.entity) {
        commands.trigger(RecalculateStats {
            entity: equipped_to.0,
        });
    }
}

fn on_item_modifiers_replace(
    ev: On<Replace, ItemModifiers>,
    mut commands: Commands,
    q_equipped_to: Query<&InInventoryOf>,
) {
    if let Ok(equipped_to) = q_equipped_to.get(ev.entity) {
        commands.trigger(RecalculateStats {
            entity: equipped_to.0,
        });
    }
}

fn on_recalculate_stats(
    ev: On<RecalculateStats>,
    mut q_owner: Query<(&BaseStats, &UpgradeStats, &mut DerivedStats)>,
    inventory: Query<&Inventory>,
    q_mods: Query<&ItemModifiers>,
) {
    let Ok((base, upgrades, mut derived)) = q_owner.get_mut(ev.entity) else {
        return;
    };

    // add starts from base + upgrades
    let mut add = base.0 + upgrades.0;

    // mul starts from 1.0
    let mut mul = Stats::one();

    for inventory_slot in inventory.iter_descendants(ev.entity) {
        if let Ok(mods) = q_mods.get(inventory_slot) {
            add += mods.add;
            mul *= mods.mul;
        }
    }

    derived.0 = (add * mul).clamp_rules();
}
