use avian2d::prelude::*;
use bevy::prelude::*;

use crate::GameLayer;

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

#[derive(Component, Reflect)]
pub(crate) struct ExplosionRadius(pub f32);

#[derive(Component, Reflect, Default)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct BaseDamage(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Range(pub f32);

#[derive(Component, Reflect)]
pub(crate) struct Halt;

#[derive(Component, Reflect)]
pub(crate) struct Root(pub Timer);
