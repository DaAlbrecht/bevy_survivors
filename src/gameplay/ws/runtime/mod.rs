use crate::gameplay::ws::{
    behaviours::shared::BaseDamage,
    prelude::*,
    runtime::visuals::{WeaponImpactVisuals, WeaponProjectileVisuals},
};
use bevy::prelude::*;

pub(crate) mod attacks;
pub(crate) mod sfx;
pub(crate) mod visuals;

pub trait ApplySpec {
    fn apply(&self, commands: &mut Commands, entity: Entity);
}

impl ApplySpec for WeaponSpec {
    fn apply(&self, commands: &mut Commands, weapon: Entity) {
        commands.entity(weapon).insert((
            Weapon,
            Name::new(self.name.clone()),
            BaseDamage(self.base_damage),
            WeaponCooldown(Timer::from_seconds(self.cooldown, TimerMode::Once)),
            WeaponProjectileVisuals(self.visuals.clone()),
        ));

        if let Some(impact) = &self.impact_visuals {
            commands
                .entity(weapon)
                .insert(WeaponImpactVisuals(impact.clone()));
        }

        self.attack.apply(commands, weapon);
        self.on_hit.apply(commands, weapon);
        self.sfx.apply(commands, weapon);
    }
}
