use crate::gameplay::ws::prelude::*;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{PausableSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_projectile)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

fn move_projectile(
    weapons: Query<(Entity, &ProjectileSpeed), With<super::ShotAttack>>,
    projectiles: Query<&WeaponProjectiles>,
    mut projectile_q: Query<
        (&mut LinearVelocity, &ProjectileDirection, Option<&Halt>),
        With<PlayerProjectile>,
    >,
) {
    // Loop over all types of weapons
    for (weapon, speed) in &weapons {
        // Iterate over each projectile for this given weapon type

        for projectile in projectiles.iter_descendants(weapon) {
            let Ok((mut linear_velocity, bullet_direction, halt)) =
                projectile_q.get_mut(projectile)
            else {
                continue;
            };

            if halt.is_some() {
                linear_velocity.0.x = 0.0;
                linear_velocity.0.y = 0.0;
                continue;
            }

            let movement = bullet_direction.0.normalize_or_zero() * speed.0;
            linear_velocity.0.x = movement.x;
            linear_velocity.0.y = movement.y;
        }
    }
}
