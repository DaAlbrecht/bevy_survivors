use crate::gameplay::weapons::{
    ApplySpec,
    behaviours::orbiters::OrbitRadius,
    prelude::{OrbitersSpec, ProjectileCount, WeaponLifetime},
};
use bevy::prelude::*;

impl ApplySpec for OrbitersSpec {
    fn apply(&self, commands: &mut Commands, entity: Entity) {
        let mut ec = commands.entity(entity);
        ec.insert((
            super::OrbitersAttack,
            ProjectileCount(self.count),
            OrbitRadius(self.radius),
            super::OrbitAngularSpeed(self.angular_speed),
            WeaponLifetime(self.lifetime),
        ));
    }
}
