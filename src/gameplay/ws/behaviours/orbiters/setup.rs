use crate::gameplay::ws::{behaviours::orbiters::OrbitRadius, prelude::*};
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
            BaseDamage(self.damage),
        ));
    }
}
