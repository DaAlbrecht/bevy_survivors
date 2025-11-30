use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use rand::Rng;

use crate::{
    GameLayer, PausableSystems,
    audio::SfxPool,
    gameplay::{
        damage_numbers::DamageType,
        enemy::{Enemy, EnemyDamageEvent},
        player::Player,
        simple_animation::{AnimationIndices, AnimationTimer},
        spells::{
            CastSpell, Cooldown, Damage, ProjectileCount, Spell, SpellType, UpgradeSpellEvent,
        },
    },
    screens::Screen,
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Circles,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    Damage(3.),
    ProjectileCount(4.),
    Name::new("Circles Spell")
)]
#[derive(Reflect)]
pub(crate) struct Circles;

#[derive(Component, Reflect)]
pub(crate) struct CircleProjectile;

#[derive(Event, Reflect)]
pub(crate) struct CirclesAttackEvent;

#[derive(Component)]
struct ZigZagMovement {
    base_direction: Vec2,
    zigzag_timer: Timer,
    zigzag_offset: f32,
    zigzag_speed: f32,
    current_target: Option<Entity>,
}

#[derive(Component)]
struct CircleHitCounter {
    hits: usize,
    max_hits: usize,
}

#[derive(Component)]
struct CircleLifetime(Timer);

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (update_zigzag_movement, update_circle_lifetime)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(spawn_circles);
    app.add_observer(on_circle_hit);
}

pub fn upgrade_circles(
    _trigger: On<UpgradeSpellEvent>,
    mut circles_q: Query<&mut ProjectileCount, With<Circles>>,
) -> Result {
    let mut count = circles_q.single_mut()?;
    count.0 += 2.0;
    info!("Circles count upgraded to: {}", count.0);

    Ok(())
}

fn spawn_circles(
    _trigger: On<CirclesAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    circles_q: Query<(Entity, &ProjectileCount), With<Circles>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) -> Result {
    let Ok(player_transform) = player_q.single() else {
        return Ok(());
    };

    let (circles, projectile_count) = circles_q.single()?;

    let texture = asset_server.load("fx/area.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(29, 26), 10, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);

    let num_circles = projectile_count.0 as usize;
    let angle_step = std::f32::consts::TAU / num_circles as f32;

    let mut rng = rand::rng();

    for i in 0..num_circles {
        let angle = angle_step * i as f32;
        let base_direction = Vec2::new(angle.cos(), angle.sin());

        let zigzag_speed = rng.random_range(30.0..60.0);
        let zigzag_frequency = rng.random_range(0.3..0.6);

        commands
            .spawn((
                Name::new("circle projectile"),
                Sprite::from_atlas_image(
                    texture.clone(),
                    TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 0,
                    },
                ),
                AnimationIndices { first: 0, last: 9 },
                AnimationTimer {
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                CastSpell(circles),
                Transform::from_xyz(
                    player_transform.translation.x,
                    player_transform.translation.y,
                    10.0,
                ),
                CircleProjectile,
            ))
            .insert((
                RigidBody::Kinematic,
                Collider::rectangle(32.0, 32.0),
                Sensor,
                CollisionEventsEnabled,
                CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
                LinearVelocity(Vec2::ZERO),
            ))
            .insert((
                ZigZagMovement {
                    base_direction,
                    zigzag_timer: Timer::from_seconds(zigzag_frequency, TimerMode::Repeating),
                    zigzag_offset: 0.0,
                    zigzag_speed,
                    current_target: None,
                },
                CircleHitCounter {
                    hits: 0,
                    max_hits: 5,
                },
                CircleLifetime(Timer::from_seconds(10.0, TimerMode::Once)),
            ))
            .observe(on_circle_hit);
    }

    Ok(())
}

fn update_zigzag_movement(
    mut circle_q: Query<
        (&Transform, &mut LinearVelocity, &mut ZigZagMovement),
        With<CircleProjectile>,
    >,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time<Fixed>>,
) {
    for (circle_transform, mut velocity, mut zigzag) in &mut circle_q {
        zigzag.zigzag_timer.tick(time.delta());

        if zigzag.zigzag_timer.just_finished() {
            zigzag.zigzag_offset = if zigzag.zigzag_offset > 0.0 {
                -1.0
            } else {
                1.0
            };
        }

        let mut closest_enemy: Option<(Entity, Vec2)> = None;
        let mut min_distance = f32::MAX;

        for (enemy_entity, enemy_transform) in &enemy_q {
            let distance = circle_transform
                .translation
                .truncate()
                .distance(enemy_transform.translation.truncate());

            if distance < min_distance && distance < 300.0 {
                min_distance = distance;
                closest_enemy = Some((enemy_entity, enemy_transform.translation.truncate()));
            }
        }

        if let Some((enemy_entity, enemy_pos)) = closest_enemy {
            if zigzag.current_target != Some(enemy_entity) {
                zigzag.current_target = Some(enemy_entity);
            }

            let to_enemy = (enemy_pos - circle_transform.translation.truncate()).normalize();
            zigzag.base_direction = (zigzag.base_direction * 0.95 + to_enemy * 0.05).normalize();
        }

        let perpendicular = Vec2::new(-zigzag.base_direction.y, zigzag.base_direction.x);

        let movement = zigzag.base_direction * 80.0
            + perpendicular * zigzag.zigzag_offset * zigzag.zigzag_speed;

        velocity.0 = movement;
    }
}

fn update_circle_lifetime(
    mut circle_q: Query<(Entity, &mut CircleLifetime), With<CircleProjectile>>,
    time: Res<Time<Fixed>>,
    mut commands: Commands,
) {
    for (entity, mut lifetime) in &mut circle_q {
        lifetime.0.tick(time.delta());

        if lifetime.0.just_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

fn on_circle_hit(
    event: On<CollisionStart>,
    enemy_q: Query<Entity, With<Enemy>>,
    mut circle_q: Query<
        (
            &CastSpell,
            &mut CircleHitCounter,
            &mut ZigZagMovement,
            &mut Sprite,
        ),
        With<CircleProjectile>,
    >,
    spell_q: Query<&Damage, With<Circles>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) -> Result {
    let circle = event.collider1;
    let enemy = event.collider2;

    let colors = [
        Color::srgb(0.5, 0.8, 1.0),
        Color::srgb(1.0, 0.5, 0.8),
        Color::srgb(0.8, 1.0, 0.5),
        Color::srgb(1.0, 0.9, 0.2),
        Color::srgb(0.7, 0.5, 1.0),
        Color::srgb(1.0, 0.3, 0.3),
    ];

    let Ok((cast_spell, mut hit_counter, mut zigzag, mut sprite)) = circle_q.get_mut(circle) else {
        return Ok(());
    };

    let Ok(dmg) = spell_q.get(cast_spell.0) else {
        return Ok(());
    };

    if let Ok(enemy) = enemy_q.get(enemy) {
        commands.trigger(EnemyDamageEvent {
            entity_hit: enemy,
            dmg: dmg.0,
            damage_type: DamageType::Physical,
        });

        commands.spawn((
            SamplePlayer::new(asset_server.load("audio/sound_effects/generic_crisp.wav")),
            SfxPool,
        ));

        hit_counter.hits += 1;

        if hit_counter.hits < colors.len() {
            sprite.color = colors[hit_counter.hits];
        }

        zigzag.current_target = None;

        if hit_counter.hits >= hit_counter.max_hits {
            commands.entity(circle).try_despawn();
        }
    }

    Ok(())
}
