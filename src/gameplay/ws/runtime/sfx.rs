use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;

#[derive(Component, Clone)]
pub struct WeaponAttackSfx(pub Handle<AudioSample>);

#[derive(Component, Clone)]
pub struct WeaponImpactSfx(pub Handle<AudioSample>);

impl ApplySpec for WeaponSfx {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        if let Some(h) = &self.attack {
            ec.insert(WeaponAttackSfx(h.clone()));
        }
        if let Some(h) = &self.impact {
            ec.insert(WeaponImpactSfx(h.clone()));
        }
    }
}
