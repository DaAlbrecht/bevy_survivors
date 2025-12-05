use crate::gameplay::abilities::{
    Ability, AbilityAssets, AbilityCooldown, UseAbility, init_ability_assets, try_use_ability,
};
use crate::gameplay::character_controller::CharacterController;
use crate::gameplay::enemy::{Enemy, EnemyDamageEvent, EnemyType};
use crate::gameplay::player::{Direction, Player};
use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::gameplay::{Health, Speed};
use crate::{GameLayer, PausableSystems, PostPhysicsAppSystems};
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
#[require(
    Ability,
    AbilityAssets,
    AbilityCooldown(AbilityCooldown::ready(20.)),
    Name::new("Summon")
)]
#[derive(Reflect)]
pub(crate) struct Summon;

#[derive(Component, Reflect)]
pub(crate) struct Minion {
    pub owner: Entity,
    pub formation_offset: Vec3,
}

#[derive(Component, Default, Reflect)]
pub(crate) struct MinionAttackCooldown(pub Timer);

#[derive(Component)]
pub(crate) struct MinionProjectile;

#[derive(Component, Default, Reflect)]
pub(crate) struct MinionLifetime(pub Timer);

#[derive(Component)]
pub(crate) struct SeekingExplosion {
    pub target_position: Vec3,
    pub speed: f32,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_use_summon);
    app.add_observer(init_ability_assets::<Summon>("ui/icons/summon_spell.png"));
    app.add_systems(
        Update,
        (
            despawn_far_projectiles,
            update_minion_animation.in_set(PostPhysicsAppSystems::PlayAnimations),
        )
            .in_set(PausableSystems),
    );
    app.add_systems(
        FixedUpdate,
        (
            minion_follow_player,
            minion_attack_enemies,
            minion_lifetime_system,
            move_seeking_minions,
        )
            .in_set(PausableSystems),
    );
}

fn on_use_summon(
    trigger: On<UseAbility>,
    mut summon_q: Query<&mut AbilityCooldown, With<Summon>>,
    player_q: Query<(Entity, &Transform), With<Player>>,
    enemy_q: Query<(Entity, &Transform, &EnemyType), With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !try_use_ability::<Summon>(trigger.ability_entity, &mut summon_q) {
        return;
    }

    let Ok((player_entity, player_transform)) = player_q.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    let mut walker_enemies: Vec<(Entity, &Transform, f32)> = enemy_q
        .iter()
        .filter(|(_, _, enemy_type)| **enemy_type == EnemyType::Walker)
        .map(|(entity, transform, _)| {
            let distance = player_pos.distance(transform.translation);
            (entity, transform, distance)
        })
        .collect();

    walker_enemies.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    let closest_walkers: Vec<_> = walker_enemies.into_iter().take(3).collect();

    if closest_walkers.is_empty() {
        return;
    }

    let texture: Handle<Image> = asset_server.load("enemies/walker.png");
    let layout = TextureAtlasLayout::from_grid(UVec2 { x: 58, y: 24 }, 11, 1, None, None);
    let texture_atlas_layout_handle = texture_atlas_layout.add(layout);

    let formation_offsets = [
        Vec3::new(40.0, 0.0, 0.0),
        Vec3::new(-20.0, 35.0, 0.0),
        Vec3::new(-20.0, -35.0, 0.0),
    ];

    for (i, (enemy_entity, enemy_transform, _)) in closest_walkers.into_iter().enumerate() {
        let enemy_pos = enemy_transform.translation;
        let spawn_position = Vec3::new(enemy_pos.x, enemy_pos.y, 10.0);

        let formation_offset = formation_offsets[i % formation_offsets.len()];

        commands.entity(enemy_entity).despawn();

        let minion = commands
            .spawn((
                Minion {
                    owner: player_entity,
                    formation_offset,
                },
                Direction::default(),
                Health(50.0),
                Speed(80.0),
                CharacterController {
                    speed: 80.0,
                    ability_velocity: Vec2::ZERO,
                },
                MinionAttackCooldown(Timer::from_seconds(2.0, TimerMode::Repeating)),
                MinionLifetime(Timer::from_seconds(10.0, TimerMode::Once)),
                LockedAxes::ROTATION_LOCKED,
                RigidBody::Dynamic,
                Collider::circle(12.),
                Friction::ZERO,
                Name::new("Minion"),
            ))
            .id();

        commands.entity(minion).insert((
            CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout_handle.clone(),
                    index: 0,
                }),
                color: Color::srgb(0.5, 1.0, 0.5),
                ..default()
            },
            AnimationIndices { first: 0, last: 10 },
            AnimationTimer {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            Transform::from_translation(spawn_position),
        ));
    }
}

fn minion_follow_player(
    player_q: Query<&Transform, With<Player>>,
    mut minion_q: Query<
        (
            &Minion,
            &Transform,
            &mut LinearVelocity,
            &mut Direction,
            &CharacterController,
        ),
        Without<Player>,
    >,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (minion, minion_transform, mut velocity, mut direction, controller) in &mut minion_q {
        let minion_pos = minion_transform.translation;
        let target_pos = player_pos + minion.formation_offset;
        let to_target = target_pos - minion_pos;
        let distance = to_target.length();

        if distance > 20.0 {
            let dir = to_target.normalize();
            let vel = dir * controller.speed;
            velocity.x = vel.x;
            velocity.y = vel.y;
            direction.0 = dir;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
            direction.0 = Vec3::ZERO;
        }
    }
}

fn minion_attack_enemies(
    mut minion_q: Query<(Entity, &Transform, &mut MinionAttackCooldown), With<Minion>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (_minion_entity, minion_transform, mut attack_cooldown) in &mut minion_q {
        attack_cooldown.0.tick(time.delta());

        if !attack_cooldown.0.is_finished() {
            continue;
        }

        let minion_pos = minion_transform.translation;

        let attack_range = 150.0;
        let mut min_distance = f32::MAX;
        let mut closest_enemy: Option<&Transform> = None;

        for enemy_transform in &enemy_q {
            let distance = minion_pos.distance(enemy_transform.translation);

            if distance <= attack_range && distance < min_distance {
                min_distance = distance;
                closest_enemy = Some(enemy_transform);
            }
        }

        if let Some(enemy_transform) = closest_enemy {
            let direction = (enemy_transform.translation - minion_pos)
                .truncate()
                .normalize();

            let texture = asset_server.load("fx/fireball.png");
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 4, 1, None, None);
            let texture_atlas_layout_handle = texture_atlas_layout.add(layout);
            let animation_indices = AnimationIndices { first: 0, last: 3 };

            let towards_quaternion =
                Quat::from_rotation_arc(Vec3::Y, direction.extend(0.).normalize());

            commands
                .spawn((
                    Name::new("minion projectile"),
                    MinionProjectile,
                    Direction(direction.extend(0.)),
                    Speed(300.0),
                    LinearVelocity(direction * 300.0),
                    LockedAxes::ROTATION_LOCKED,
                    RigidBody::Kinematic,
                    Collider::circle(8.),
                    CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy]),
                    CollisionEventsEnabled,
                    Sprite {
                        image: texture,
                        texture_atlas: Some(TextureAtlas {
                            layout: texture_atlas_layout_handle,
                            index: animation_indices.first,
                        }),
                        color: Color::srgb(0.0, 1.0, 0.3),
                        ..default()
                    },
                    animation_indices,
                    AnimationTimer {
                        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    },
                    Transform::from_xyz(minion_pos.x, minion_pos.y, 10.0)
                        .with_rotation(towards_quaternion),
                ))
                .observe(on_minion_projectile_hit);

            attack_cooldown.0.reset();
        }
    }
}

fn on_minion_projectile_hit(
    event: On<CollisionStart>,
    enemy_q: Query<Entity, With<Enemy>>,
    projectile_q: Query<&MinionProjectile>,
    mut commands: Commands,
) {
    let projectile = event.collider1;
    let enemy = event.collider2;

    if enemy_q.get(enemy).is_ok() && projectile_q.get(projectile).is_ok() {
        commands.trigger(EnemyDamageEvent {
            entity_hit: enemy,
            dmg: 2.0,
            damage_type: crate::gameplay::damage_numbers::DamageType::Physical,
        });

        commands.entity(projectile).despawn();
    }
}

fn minion_lifetime_system(
    mut minion_q: Query<
        (Entity, &Minion, &Transform, &mut MinionLifetime),
        Without<SeekingExplosion>,
    >,
    enemy_q: Query<&Transform, (With<Enemy>, Without<Minion>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, minion, transform, mut lifetime) in &mut minion_q {
        lifetime.0.tick(time.delta());

        if lifetime.0.is_finished() {
            let minion_pos = transform.translation;

            let preferred_direction = minion.formation_offset.truncate().normalize_or_zero();
            let mut best_enemy: Option<Vec3> = None;
            let mut best_score = f32::MIN;

            for enemy_transform in &enemy_q {
                let to_enemy = enemy_transform.translation - minion_pos;
                let distance = to_enemy.length();

                if distance > 0.1 {
                    let direction = to_enemy.normalize();
                    let alignment = direction.truncate().dot(preferred_direction);
                    let score = alignment - (distance / 200.0);

                    if score > best_score {
                        best_score = score;
                        best_enemy = Some(enemy_transform.translation);
                    }
                }
            }

            let target_position = if let Some(enemy_pos) = best_enemy {
                enemy_pos
            } else {
                minion_pos + minion.formation_offset * 3.0
            };

            commands.entity(entity).remove::<MinionLifetime>();
            commands.entity(entity).remove::<MinionAttackCooldown>();
            commands.entity(entity).insert(SeekingExplosion {
                target_position,
                speed: 150.0,
            });
        }
    }
}

fn move_seeking_minions(
    mut seeking_q: Query<(Entity, &mut Transform, &SeekingExplosion), With<Minion>>,
    enemy_q: Query<(Entity, &Transform), (With<Enemy>, Without<Minion>)>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, mut transform, seeking) in &mut seeking_q {
        let current_pos = transform.translation;
        let to_target = seeking.target_position - current_pos;
        let distance = to_target.length();

        if distance < 10.0 {
            spawn_minion_death_effect(
                current_pos,
                &mut commands,
                &asset_server,
                &mut texture_atlas_layout,
                &enemy_q,
            );
            commands.entity(entity).despawn();
        } else {
            let direction = to_target.normalize();
            let movement = direction * seeking.speed * time.delta_secs();
            transform.translation += movement;
        }
    }
}

fn spawn_minion_death_effect(
    position: Vec3,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: &mut ResMut<Assets<TextureAtlasLayout>>,
    enemy_q: &Query<(Entity, &Transform), (With<Enemy>, Without<Minion>)>,
) {
    let explosion_radius = 100.0;
    let explosion_damage = 10.0;

    for (enemy_entity, enemy_transform) in enemy_q.iter() {
        let distance = position.distance(enemy_transform.translation);

        if distance <= explosion_radius {
            commands.trigger(EnemyDamageEvent {
                entity_hit: enemy_entity,
                dmg: explosion_damage,
                damage_type: crate::gameplay::damage_numbers::DamageType::Physical,
            });
        }
    }

    let texture = asset_server.load("fx/fireball_hit.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(25, 30), 8, 1, None, None);
    let texture_atlas_layout_handle = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 7 };

    commands.spawn((
        Name::new("Minion Death Explosion"),
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout_handle,
                index: animation_indices.first,
            }),
            color: Color::srgb(0.2, 1.0, 0.4),
            ..default()
        },
        animation_indices,
        AnimationTimer::once_from_fps(24),
        Transform::from_xyz(position.x, position.y, 10.0).with_scale(Vec3::splat(3.0)),
    ));

    let flash_texture = asset_server.load("fx/fireball_hit.png");
    let flash_layout = TextureAtlasLayout::from_grid(UVec2::new(25, 30), 8, 1, None, None);
    let flash_layout_handle = texture_atlas_layout.add(flash_layout);
    let flash_indices = AnimationIndices { first: 0, last: 7 };

    commands.spawn((
        Name::new("Minion Death Flash"),
        Sprite {
            image: flash_texture,
            texture_atlas: Some(TextureAtlas {
                layout: flash_layout_handle,
                index: flash_indices.first,
            }),
            color: Color::srgba(0.5, 1.0, 0.6, 0.6),
            ..default()
        },
        flash_indices,
        AnimationTimer::once_from_fps(30),
        Transform::from_xyz(position.x, position.y, 10.1).with_scale(Vec3::splat(4.5)),
    ));
}

fn update_minion_animation(mut minion_q: Query<(&Direction, &mut Sprite), With<Minion>>) {
    for (direction, mut sprite) in &mut minion_q {
        let dx = direction.0.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }
    }
}

fn despawn_far_projectiles(
    projectile_q: Query<(Entity, &Transform), With<MinionProjectile>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (entity, projectile_transform) in &projectile_q {
        let distance = player_pos.distance(projectile_transform.translation);

        if distance > 500.0 {
            commands.entity(entity).despawn();
        }
    }
}
