use crate::gameplay::weapons::prelude::*;
use bevy::prelude::*;

/// A player has attacked with a Weapon
///
/// The Entity that this Event is triggered for is the Weapon that
/// attacked.
#[derive(EntityEvent)]
pub struct WeaponAttackEvent {
    /// The Weapon that attacked.
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
