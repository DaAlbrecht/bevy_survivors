use crate::{
    GameLayer,
    gameplay::{
        player::{Player, PlayerFacing},
        simple_animation::{AnimationIndices, AnimationPlayback, AnimationTimer},
        weapons::{
            behaviours::{WeaponProjectileVisuals, melee::MeleeAttackZone},
            components::CastWeapon,
            systems::{attack::WeaponAttackEvent, cooldown::WeaponDuration},
        },
    },
};
use avian2d::prelude::*;
use bevy::prelude::*;

const DEFAUL_DURATION: f32 = 0.5;

pub fn on_melee_attack(
    trigger: On<WeaponAttackEvent>,
    weapon_q: Query<(&super::AttackCone, &WeaponProjectileVisuals), With<super::MeleeAttack>>,
    player_q: Query<(&Transform, &PlayerFacing), With<Player>>,
    mut commands: Commands,
) -> Result {
    let weapon = trigger.event().entity;

    let Ok((cone, visuals)) = weapon_q.get(weapon) else {
        return Ok(());
    };

    let (player_pos, facing) = player_q.single()?;

    let facing_right = facing.is_right();
    let dir_x = if facing_right { 1.0 } else { -1.0 };

    let angle = if facing_right {
        0.0
    } else {
        std::f32::consts::PI
    };

    let half_angle = cone.angle.to_radians() / 2.0;
    let left_direction = Vec2::from_angle(angle + half_angle);
    let right_direction = Vec2::from_angle(angle - half_angle);

    let apex = Vec2::ZERO;
    let left_point = left_direction * cone.range;
    let right_point = right_direction * cone.range;
    let vec: Vec<Vec2> = vec![apex, left_point, right_point];

    let collider = Collider::convex_hull(vec).expect("Can create cone collider");

    let duration_secs = visuals.0.duration_secs_once_or(DEFAUL_DURATION);
    let proj = commands
        .spawn((
            Name::new("MeleeAttackCone"),
            MeleeAttackZone,
            CastWeapon(weapon),
            collider,
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
            DebugRender::default().with_collider_color(Color::srgb(0.0, 1.0, 0.0)),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 10.0),
            WeaponDuration(Timer::from_seconds(duration_secs, TimerMode::Once)),
        ))
        .id();

    //TODO: SET PER WEAPON
    let sprite_offset_x = cone.range * 0.5;

    commands.entity(proj).with_children(|c| {
        let mut sprite = visuals.0.get_sprite();
        sprite.flip_x = !facing_right;

        let mut child = c.spawn((
            Name::new("MeleeAttackConeVisual"),
            sprite,
            Transform::from_xyz(dir_x * sprite_offset_x, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        if let Some(atlas) = visuals.0.atlas.as_ref() {
            child.insert((
                AnimationIndices {
                    first: atlas.first,
                    last: atlas.last,
                },
                AnimationTimer::from_fps(atlas.fps),
                AnimationPlayback::OnceDespawn,
            ));
        }
    });

    Ok(())
}
