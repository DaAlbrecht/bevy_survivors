use crate::gameplay::weapons::prelude::*;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{PausableSystems, screens::Screen};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        move_projectiles
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

pub(crate) fn move_projectiles(
    weapons: Query<(Entity, &ProjectileSpeed)>,
    projectiles: Query<&WeaponProjectiles>,
    mut projectile_q: Query<(&mut LinearVelocity, &ProjectileDirection), With<PlayerProjectile>>,
) {
    for (weapon, speed) in &weapons {
        for projectile in projectiles.iter_descendants(weapon) {
            let Ok((mut linear_velocity, bullet_direction)) = projectile_q.get_mut(projectile)
            else {
                continue;
            };

            let movement = bullet_direction.0.normalize_or_zero() * speed.0;
            linear_velocity.0.x = movement.x;
            linear_velocity.0.y = movement.y;
        }
    }
}
