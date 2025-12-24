use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    PausableSystems,
    gameplay::{enemy::Enemy, weapons::prelude::*},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (retarget_homing, move_homing_projectiles)
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
struct ZigzagState {
    timer: Timer,
    offset: f32,
}

#[derive(Component)]
struct SpiralState {
    angle: f32,
}

fn retarget_homing(
    mut projectiles: Query<(&Transform, &mut super::CurrentTarget), With<super::HomingProjectile>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (proj_transform, mut current_target) in &mut projectiles {
        let target_valid = current_target.0.and_then(|t| enemies.get(t).ok()).is_some();

        if !target_valid {
            let mut min_distance = f32::MAX;
            let mut closest: Option<Entity> = None;

            for (enemy, enemy_transform) in &enemies {
                let distance = proj_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance < min_distance {
                    min_distance = distance;
                    closest = Some(enemy);
                }
            }

            current_target.0 = closest;
        }
    }
}

fn move_homing_projectiles(
    time: Res<Time>,
    weapons: Query<(Entity, &ProjectileSpeed)>,
    projectiles_lookup: Query<&WeaponProjectiles>,
    enemies: Query<&Transform, With<Enemy>>,
    mut projectiles: Query<
        (
            &Transform,
            &mut LinearVelocity,
            &mut ProjectileDirection,
            &super::CurrentTarget,
            &super::MovementConfig,
            Option<&mut ZigzagState>,
            Option<&mut SpiralState>,
        ),
        With<super::HomingProjectile>,
    >,
    mut commands: Commands,
) {
    for (weapon, speed) in &weapons {
        for projectile in projectiles_lookup.iter_descendants(weapon) {
            let Ok((
                proj_transform,
                mut velocity,
                mut direction,
                target,
                movement_config,
                zigzag_state,
                spiral_state,
            )) = projectiles.get_mut(projectile)
            else {
                continue;
            };

            let base_direction = if let Some(target_entity) = target.0 {
                if let Ok(target_transform) = enemies.get(target_entity) {
                    (target_transform.translation - proj_transform.translation).normalize_or_zero()
                } else {
                    direction.0.normalize_or_zero()
                }
            } else {
                direction.0.normalize_or_zero()
            };

            let final_direction = match &movement_config.pattern {
                super::MovementPatternKind::Straight => base_direction,

                super::MovementPatternKind::Zigzag {
                    frequency,
                    amplitude,
                } => {
                    let mut state = if let Some(s) = zigzag_state {
                        s
                    } else {
                        commands.entity(projectile).insert(ZigzagState {
                            timer: Timer::from_seconds(*frequency, TimerMode::Repeating),
                            offset: 0.0,
                        });
                        continue;
                    };

                    state.timer.tick(time.delta());
                    if state.timer.just_finished() {
                        state.offset = -state.offset;
                        if state.offset == 0.0 {
                            state.offset = *amplitude;
                        }
                    }

                    let perpendicular = Vec3::new(-base_direction.y, base_direction.x, 0.0);
                    (base_direction + perpendicular * state.offset / amplitude).normalize()
                }

                super::MovementPatternKind::Wave {
                    frequency,
                    amplitude,
                } => {
                    let mut state = if let Some(s) = zigzag_state {
                        s
                    } else {
                        commands.entity(projectile).insert(ZigzagState {
                            timer: Timer::from_seconds(*frequency, TimerMode::Repeating),
                            offset: 0.0,
                        });
                        continue;
                    };

                    state.timer.tick(time.delta());
                    let wave_offset =
                        (state.timer.elapsed_secs() * std::f32::consts::TAU / frequency).sin()
                            * amplitude;

                    let perpendicular = Vec3::new(-base_direction.y, base_direction.x, 0.0);
                    (base_direction + perpendicular * wave_offset / amplitude).normalize()
                }

                super::MovementPatternKind::Spiral { rotation_speed } => {
                    let mut state = if let Some(s) = spiral_state {
                        s
                    } else {
                        commands
                            .entity(projectile)
                            .insert(SpiralState { angle: 0.0 });
                        continue;
                    };

                    state.angle += rotation_speed * time.delta_secs();

                    let rotated_x =
                        base_direction.x * state.angle.cos() - base_direction.y * state.angle.sin();
                    let rotated_y =
                        base_direction.x * state.angle.sin() + base_direction.y * state.angle.cos();

                    Vec3::new(rotated_x, rotated_y, 0.0).normalize()
                }
            };

            direction.0 = final_direction;
            let movement = final_direction * speed.0;
            velocity.0 = Vec2::new(movement.x, movement.y);
        }
    }
}
