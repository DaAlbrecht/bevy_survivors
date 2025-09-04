use bevy::prelude::*;

use crate::{
    PLAYER_SIZE, SPELL_SIZE,
    gameplay::{
        enemy::{Enemy, Speed},
        player::{Direction, Player},
        spells::{
            CastSpell, Cooldown, Damage, Halt, Knockback, PlayerProjectile, ProjectileCount, Spell,
            SpellDuration, SpellType, StartPosition,
        },
    },
    screens::Screen,
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Thorn,
    Cooldown(Timer::from_seconds(1., TimerMode::Once)),
    Speed(600.),
    Knockback(1500.),
    Damage(5.),
    ProjectileCount(5.0),
    Name::new("Thorn")
)]
pub(crate) struct Thorn;

#[derive(Component)]
pub(crate) struct ThornTip;

#[derive(Component, Default)]
pub(crate) struct ThornSegments(i32);

#[derive(Event)]
pub(crate) struct ThornAttackEvent;

#[derive(Event)]
pub(crate) struct ThornHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            thorn_range_keeper.run_if(in_state(Screen::Gameplay)),
            thorn_lifetime.run_if(in_state(Screen::Gameplay)),
        ),
    );
    app.add_observer(spawn_thorn_projectile);
}

fn spawn_thorn_projectile(
    _trigger: Trigger<ThornAttackEvent>,
    player_pos_q: Query<&Transform, With<Player>>,
    thorn_q: Query<Entity, With<Thorn>>,
    enemy_pos_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player_pos_q.single()?.translation.truncate();
    let thorn = thorn_q.single()?;

    let mut min_distance = f32::MAX;
    let mut clossest_enemy: Option<&Transform> = None;

    for enemy_pos in &enemy_pos_q {
        let distance = player_pos.distance(enemy_pos.translation.truncate());

        if distance < min_distance {
            min_distance = distance;
            clossest_enemy = Some(enemy_pos);
        }
    }

    if let Some(enemy_pos) = clossest_enemy {
        let direction = (enemy_pos.translation.truncate() - player_pos).normalize();
        let thorn_pos = player_pos + (direction * ((PLAYER_SIZE / 2.0) + (SPELL_SIZE / 2.0)));
        let angle = direction.y.atan2(direction.x);
        commands.spawn((
            Name::new("ThornTip"),
            Sprite {
                image: asset_server.load("Thorn_tip.png"),
                ..default()
            },
            CastSpell(thorn),
            Transform {
                translation: Vec3::new(thorn_pos.x, thorn_pos.y, 0.0),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            Direction(direction.extend(0.0)),
            StartPosition(Vec2::new(thorn_pos.x, thorn_pos.y)),
            ThornTip,
            ThornSegments::default(),
            PlayerProjectile,
        ));
    }
    Ok(())
}

fn thorn_range_keeper(
    thorn_q: Query<(Entity, &ProjectileCount), With<Thorn>>,
    mut thorn_tip_q: Query<
        (
            Entity,
            &Transform,
            &Direction,
            Option<&Halt>,
            &StartPosition,
            &mut ThornSegments,
        ),
        With<ThornTip>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let (thorn, thorn_count) = thorn_q.single()?;

    for (thorn_tip, transform, direction, halt, start_pos, mut segments_spawned) in &mut thorn_tip_q
    {
        let thorn_pos = transform.translation.truncate();
        //Get the angle of the thorn_tip to player
        let dir2 = direction.0.truncate();
        let distance = (thorn_pos - start_pos.0).dot(dir2);

        let offset = PLAYER_SIZE * 0.5 + SPELL_SIZE * 0.5;
        let distance_after_offset = distance - offset;

        if distance_after_offset >= segments_spawned.0 as f32 * SPELL_SIZE && halt.is_none() {
            let thorn_base = commands
                .spawn((
                    Name::new("ThornBase"),
                    Sprite {
                        image: asset_server.load("Thorn_base.png"),
                        ..default()
                    },
                    CastSpell(thorn),
                    PlayerProjectile,
                    Transform {
                        translation: Vec3::new(
                            -SPELL_SIZE * (segments_spawned.0 + 1) as f32,
                            0.0,
                            0.0,
                        ),
                        rotation: Quat::IDENTITY,
                        ..default()
                    },
                ))
                .id();

            commands.entity(thorn_tip).add_child(thorn_base);
            segments_spawned.0 += 1;
        }

        if distance >= thorn_count.0 * SPELL_SIZE && halt.is_none() {
            commands.entity(thorn_tip).insert(Halt);
            commands
                .entity(thorn_tip)
                .insert(SpellDuration(Timer::from_seconds(0.1, TimerMode::Once)));
        }
    }

    Ok(())
}

fn thorn_lifetime(
    mut thorn_tip_q: Query<(
        Entity,
        Option<&Children>,
        &mut SpellDuration,
        &mut ThornSegments,
    )>,
    mut commands: Commands,
) {
    for (thorn_tip, children, mut duration, mut segments) in &mut thorn_tip_q {
        if duration.0.finished() {
            let index = (segments.0 - 1) as usize;
            if let Some(child) = children {
                commands.entity(child[index]).despawn();
                segments.0 -= 1;
                duration.0.reset();
            } else {
                commands.entity(thorn_tip).despawn();
            }
        }
    }
}
