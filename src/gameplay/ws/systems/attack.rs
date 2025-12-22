use crate::gameplay::ws::prelude::*;
use bevy::prelude::*;

/// Weapon <entity> cooled down and is ready to perform an attack
#[derive(EntityEvent)]
pub struct WeaponAttackEvent {
    /// Weapon entity that attacked
    pub entity: Entity,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, dispatch_weapon_attacks);
}

fn dispatch_weapon_attacks(
    mut commands: Commands,
    mut q: Query<(Entity, &mut WeaponCooldown), With<Weapon>>,
) {
    for (weapon, mut cd) in &mut q {
        if cd.0.is_finished() {
            cd.0.reset();
            commands.trigger(WeaponAttackEvent { entity: weapon });
        }
    }
}
