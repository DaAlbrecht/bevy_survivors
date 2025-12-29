use bevy::prelude::*;

use crate::gameplay::{
    player::{InInventoryOf, Player},
    weapons::{
        assets::spec::WeaponSpec,
        behaviours::{WeaponImpactVisuals, WeaponProjectileVisuals},
        components::{BaseDamage, CollisionDamage, DeathOnCollision, TickDamage, Weapon},
        systems::cooldown::WeaponCooldown,
    },
};

pub(crate) mod assets;
mod behaviours;
pub(crate) mod components;
pub(crate) mod kind;
pub(crate) mod systems;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, behaviours::plugin, systems::plugin));
}

pub(crate) struct AddWeapon(WeaponSpec);

impl Command for AddWeapon {
    fn apply(self, world: &mut World) {
        let mut query = world.query_filtered::<Entity, With<Player>>();
        let Ok(player) = query.single(world) else {
            return;
        };

        let mut commands = world.commands();

        let mut entity = commands.spawn((
            Name::new(format!("{:?}", self.0.kind)),
            Weapon,
            self.0.kind,
            InInventoryOf(player),
            BaseDamage(self.0.base_damage),
            WeaponCooldown(Timer::from_seconds(self.0.cooldown, TimerMode::Once)),
            WeaponProjectileVisuals(self.0.visuals),
            //TODO: Move to attack spec to allow for pass through
            DeathOnCollision,
        ));

        entity.queue(self.0.attack);
        entity.queue(self.0.on_hit);
        entity.queue(self.0.sfx);

        match self.0.dot {
            Some(dot) => {
                entity.insert(TickDamage(Timer::from_seconds(dot, TimerMode::Repeating)));
            }
            None => {
                entity.insert(CollisionDamage);
            }
        }

        if let Some(impact) = self.0.impact_visuals {
            entity.insert(WeaponImpactVisuals(impact));
        }
    }
}
