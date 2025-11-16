use bevy::prelude::*;
use bevy_ecs_ldtk::utils::translation_to_grid_coords;

use crate::{
    PausableSystems, PhysicsAppSystems, PostPhysicsAppSystems, gameplay::level::LevelWalls,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        ((advance_physics.in_set(PhysicsAppSystems::AdvancePhysics),).in_set(PausableSystems),),
    );

    app.add_systems(
        RunFixedMainLoop,
        (interpolate_rendered_transform)
            .chain()
            .in_set(PostPhysicsAppSystems::InterpolateTransforms)
            .in_set(PausableSystems),
    );
}

/// The actual position of the player in the physics simulation.
/// This is separate from the `Transform`, which is merely a visual representation.
///
/// If you want to make sure that this component is always initialized
/// with the same value as the `Transform`'s translation, you can
/// use a [component lifecycle hook](https://docs.rs/bevy/0.14.0/bevy/ecs/component/struct.ComponentHooks.html)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct PhysicalTranslation(pub Vec3);

/// The value [`PhysicalTranslation`] had in the last fixed timestep.
/// Used for interpolation in the `interpolate_rendered_transform` system.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub(crate) struct PreviousPhysicalTranslation(pub Vec3);

pub const IMPULSE_SCALE: f32 = 0.25; // global scale applied to incoming impulses (small but snappy)
pub const BLEND_EXPONENT: f32 = 0.5; // exponent used for blending AI vs knockback (0.5 == sqrt)
pub const MIN_MASS: f32 = 1e-4;

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct MovementController {
    /// Mass of the entity (units: arbitrary, affects knockback response).
    pub mass: f32,

    /// Normal movement velocity (units: normalized direction vector).
    pub velocity: Vec3,

    /// Knockback velocity (units: world units / s). Added by apply_impulse and decayed over time.
    pub knockback_velocity: Vec3,

    /// Base resistance to knockback [0..1], 0=no resistance, 1=immune.
    pub knockback_resistance: f32,
    ///
    /// Exponential damping for knockback (higher => faster decay).
    pub knockback_damping: f32,

    /// Max Movement speed multiplier: used to turn `velocity` into world units/sec.
    pub speed: f32,

    /// CUrrent speed. We accelerate towards speed
    pub current_speed: f32,

    /// Acceleration rate towards target speed (units: world units/s²).
    pub acceleration: f32,

    /// If it should be treated as a solid object for collision purposes
    pub solid: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            mass: 1.0,
            velocity: Vec3::ZERO,
            knockback_velocity: Vec3::ZERO,
            knockback_resistance: 0.0,
            speed: 50.0,
            current_speed: 50.0,
            acceleration: 0.,
            knockback_damping: 25.0,
            solid: true,
        }
    }
}

impl MovementController {
    /// Apply a one-shot knockback impulse. The impulse is a velocity (world units/s).
    /// The actual applied impulse is reduced by knockback_resistance
    fn apply_knockback(&mut self, impulse: Vec3) {
        let mut applied = impulse * IMPULSE_SCALE;

        let resistance = self.knockback_resistance.clamp(0.0, 1.0);
        applied *= 1.0 - resistance;

        // convert to dv by dividing by mass (avoid zero)
        let mass = if self.mass > MIN_MASS {
            self.mass
        } else {
            MIN_MASS
        };
        applied *= 1.0 / mass;

        // accumulate the runtime knockback velocity
        self.knockback_velocity += applied;
    }

    /// Compute a raw impulse from the source (projectile or body) using its mass & speed.
    pub fn apply_knockback_from_source(&mut self, dir: Vec3, source: &MovementController) {
        if dir.length_squared() <= 0.0 {
            return;
        }

        let source_world_speed = (source.velocity * source.speed).length();
        let target_world_speed = (self.velocity * self.speed).length();

        let raw_mag = source.mass * (source_world_speed - target_world_speed).max(0.0);

        let raw_impulse = dir.normalize() * raw_mag;

        self.apply_knockback(raw_impulse);
    }

    /// Decay knockback exponentially with time: v *= exp(-damping * dt)
    pub fn decay_knockback(&mut self, dt: f32) {
        if self.knockback_velocity != Vec3::ZERO {
            let factor = (-(self.knockback_damping * dt)).exp();
            self.knockback_velocity *= factor;

            // threshold small residual to zero to avoid jitter
            if self.knockback_velocity.length_squared() < 1e-6 {
                self.knockback_velocity = Vec3::ZERO;
            }
        }
    }

    // /// Helper: is entity meaningfully knocked back?
    // pub fn is_knocked_back(&self, threshold: f32) -> bool {
    //     self.knockback_velocity.length_squared() > (threshold * threshold)
    // }
    //
    // /// Reset transient motion (if needed)
    // pub fn clear(&mut self) {
    //     self.knockback_velocity = Vec3::ZERO;
    //     self.velocity = Vec3::ZERO;
    // }
}

/// Marks an entity that teleported this frame, skipping interpolation.
#[derive(Component, Default)]
pub(crate) struct Teleported;

/// Here we read all movement on the controller. direct, external forces etc and calculate the new
/// position in the physics world. Avoid modyfing he physical translation directly outside of this
/// system.
fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    level_walls: Res<LevelWalls>,
    mut query: Query<(
        &mut MovementController,
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
    )>,
) {
    for (mut controller, mut current_physical_translation, mut previous_physical_translation) in
        query.iter_mut()
    {
        let dt = fixed_time.delta_secs();

        if controller.acceleration > 0.0 {
            if controller.velocity.length_squared() > 0.0 {
                controller.current_speed += controller.acceleration * dt;
                controller.current_speed = controller.current_speed.clamp(0.0, controller.speed);
            }
        } else {
            controller.current_speed = controller.speed;
        }

        let movement_vel = controller.velocity * controller.current_speed;

        let knockback_vel = controller.knockback_velocity;

        // magnitudes
        let mv_speed = movement_vel.length();
        let kb_speed = knockback_vel.length();

        // Compute a blend factor t = kb/(kb+ai), then shape with BLEND_EXPONENT.
        // BLEND_EXPONENT < 1 (e.g. 0.5) makes small kb have a stronger immediate perceptible effect.
        let blend = if (kb_speed + mv_speed) > 0.0 {
            let t = (kb_speed / (kb_speed + mv_speed)).clamp(0.0, 1.0);
            t.powf(BLEND_EXPONENT)
        } else {
            0.0
        };

        let final_velocity = movement_vel.lerp(knockback_vel, blend);

        let desired_delta = final_velocity * dt;

        let allowed_delta = if controller.solid {
            resolve_grid_collision(
                current_physical_translation.0,
                desired_delta,
                &level_walls,
                32.0, // tile size
            )
        } else {
            desired_delta
        };

        previous_physical_translation.0 = current_physical_translation.0;
        current_physical_translation.0 += allowed_delta;

        controller.decay_knockback(dt);
    }
}

pub fn resolve_grid_collision(
    current_pos: Vec3,
    desired_delta: Vec3,
    walls: &LevelWalls,
    tile_size: f32,
) -> Vec3 {
    let mut allowed = desired_delta;

    // --- X axis ---
    if desired_delta.x != 0.0 {
        let next = current_pos + Vec3::new(desired_delta.x, 0.0, 0.0);
        let next_cell = translation_to_grid_coords(next.truncate(), IVec2::splat(tile_size as i32));

        if walls.in_wall(&next_cell) {
            allowed.x = 0.0;
        }
    }

    // --- Y axis ---
    if desired_delta.y != 0.0 {
        let next = current_pos + Vec3::new(0.0, desired_delta.y, 0.0);
        let next_cell = translation_to_grid_coords(next.truncate(), IVec2::splat(tile_size as i32));

        if walls.in_wall(&next_cell) {
            allowed.y = 0.0;
        }
    }

    allowed
}

fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &mut Transform,
        Option<&Teleported>,
        Entity,
    )>,
    mut commands: Commands,
) {
    for (
        current_physical_translation,
        previous_physical_translation,
        mut transform,
        teleported,
        entity,
    ) in &mut query
    {
        if teleported.is_some() {
            // Snap instantly if entity teleported
            transform.translation = current_physical_translation.0;
            commands.entity(entity).remove::<Teleported>();
        } else {
            let previous = previous_physical_translation.0;
            let current = current_physical_translation.0;
            // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
            let alpha = fixed_time.overstep_fraction();

            let rendered_translation = previous.lerp(current, alpha);
            transform.translation = rendered_translation;
        }
    }
}
