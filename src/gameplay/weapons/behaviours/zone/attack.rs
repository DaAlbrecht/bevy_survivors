use crate::{
    GameLayer,
    gameplay::{
        enemy::Enemy,
        player::Player,
        weapons::{
            behaviours::{
                WeaponProjectileVisuals,
                zone::{ZoneShape, ZoneTarget},
            },
            components::{CastWeapon, WeaponLifetime},
            systems::{attack::WeaponAttackEvent, cooldown::WeaponDuration},
        },
    },
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn on_zone_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<
        (
            &ZoneShape,
            &ZoneTarget,
            &WeaponLifetime,
            &WeaponProjectileVisuals,
        ),
        With<super::ZoneAttack>,
    >,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((shape, zone_target, lifetime, visuals)) = weapon_q.get(weapon) else {
        return Ok(());
    };

    let player_pos = player_q.single()?;
    let Some(target) = get_target_position(zone_target, player_pos, &enemy_q) else {
        return Ok(());
    };

    let sprite_size = visuals.0.size;

    let (scale, collider) = match shape {
        super::ZoneShape::Circle { radius } => {
            let diameter = radius * 2.0;
            let visual_scale = diameter / sprite_size.x;
            let scale = Vec2::splat(visual_scale);
            let collider = Collider::circle(radius / visual_scale);

            (scale, collider)
        }
    };

    let mut proj = commands.spawn((
        Name::new("ZoneAttackInstance"),
        super::ZoneAttackInstance,
        CastWeapon(weapon),
        collider,
        Sensor,
        CollisionEventsEnabled,
        CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
        DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
        Transform {
            translation: Vec3::new(target.translation.x, target.translation.y, -1.0),
            scale: scale.extend(1.0),
            ..default()
        },
        WeaponDuration(Timer::from_seconds(lifetime.0, TimerMode::Once)),
    ));

    visuals.0.apply_ec(&mut proj);

    Ok(())
}

fn get_target_position(
    target: &super::ZoneTarget,
    player_pos: &Transform,
    enemy_q: &Query<&Transform, With<Enemy>>,
) -> Option<Transform> {
    match target {
        super::ZoneTarget::Player => Some(*player_pos),
        super::ZoneTarget::Enemy => {
            let mut min_distance = f32::MAX;
            let mut closest_enemy: Option<Transform> = None;

            for enemy_pos in enemy_q.iter() {
                let distance = player_pos
                    .translation
                    .truncate()
                    .distance(enemy_pos.translation.truncate());

                if distance < min_distance {
                    min_distance = distance;
                    closest_enemy = Some(*enemy_pos);
                }
            }

            closest_enemy
        }
    }
}
