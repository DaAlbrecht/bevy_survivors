use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    GameLayer,
    gameplay::{damage_numbers::DamageType, weapons::systems::cooldown::WeaponDuration},
};

#[derive(Component, Reflect)]
#[require(
    RigidBody::Kinematic,
    Collider= Collider::rectangle(16., 16.) ,
    DebugRender = DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
    CollisionEventsEnabled,
    CollisionLayers =  CollisionLayers::new(
        GameLayer::Player,
        [
            GameLayer::Enemy,
            GameLayer::Default,
        ],
    ),
    // TODO: This is a placeholder to remove projectiles after some time.
    WeaponDuration(Timer::from_seconds(5.0, TimerMode::Once)),
)]
pub(crate) struct PlayerProjectile;

#[derive(Component)]
#[relationship(relationship_target = WeaponProjectiles)]
#[derive(Reflect)]
pub(crate) struct CastWeapon(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CastWeapon, linked_spawn)]
#[derive(Reflect)]
pub(crate) struct WeaponProjectiles(Vec<Entity>);

#[derive(Component, Reflect, Default)]
pub(crate) struct ProjectileDirection(pub Vec3);

#[derive(Component, Reflect, Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct ProjectileCount(pub u32);

#[derive(Component, Reflect)]
pub(crate) struct WeaponLifetime(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct WeaponRange(pub f32);

#[derive(Component, Reflect)]
pub struct DeathOnCollision;

#[derive(Component, Reflect)]
pub struct CollisionDamage;

#[derive(Component, Reflect)]
pub struct TickDamageTimer(pub Timer);

#[derive(Component, Reflect)]
pub(crate) struct ExplosionRadius(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct BaseDamage(pub f32);

#[derive(Component, Reflect)]
pub struct TickDuration(pub f32);

#[derive(Component, Clone, Reflect)]
pub(crate) struct DoT {
    pub duration: Timer,
    pub tick: Timer,
    pub dmg_per_tick: f32,
    pub damage_type: DamageType,
}
