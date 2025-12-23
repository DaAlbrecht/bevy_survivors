use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

use crate::audio::SfxPool;
use crate::gameplay::damage_numbers::DamageType;
use crate::gameplay::player::{Direction, Level};
use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::gameplay::weapons::{ExplosionRadius, Range, UpgradeWeaponEvent, WeaponAttackEvent};
use crate::gameplay::{
    Speed,
    enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
    player::Player,
    weapons::{CastWeapon, Cooldown, Damage, Knockback, PlayerProjectile, Weapon, WeaponType},
};

#[derive(Component)]
#[require(
    Weapon,
    WeaponType::Fireball,
    Level(1.0),
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    Speed(600.),
    Knockback(100.),
    Damage(5.),
    ExplosionRadius(100.),
    Range(200.),
    Name::new("Fireball")
)]
#[derive(Reflect)]
pub(crate) struct Fireball;

#[derive(Event, Reflect)]
pub(crate) struct FireballAttackEvent;

// pub(crate) fn plugin(app: &mut App) {}

pub fn upgrade_fireball(
    _trigger: On<UpgradeWeaponEvent>,
    mut fireball_q: Query<(&mut Level, &mut Damage), With<Fireball>>,
) -> Result {
    let (mut level, mut damage) = fireball_q.single_mut()?;

    level.0 += 1.0;
    damage.0 = level.0 * 5.0;
    info!("Fireball damage upgraded to: {}", damage.0);

    Ok(())
}

pub fn spawn_fireball_projectile(
    _trigger: On<WeaponAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    fireball: Query<Entity, With<Fireball>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let fireball = fireball.single()?;

    let mut min_distance = f32::MAX;
    let mut closest_enemy: Option<&Transform> = None;

    for enemy_pos in &enemy_q {
        let distance = player_pos
            .translation
            .truncate()
            .distance(enemy_pos.translation.truncate());

        if distance < min_distance {
            min_distance = distance;
            closest_enemy = Some(enemy_pos);
        }
    }

    let texture = asset_server.load("fx/fireball.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 4, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    if let Some(enemy_pos) = closest_enemy {
        let direction = (enemy_pos.translation - player_pos.translation)
            .truncate()
            .normalize();

        let towards_quaternion = Quat::from_rotation_arc(Vec3::Y, direction.extend(0.).normalize());

        commands
            .spawn((
                Name::new("fireball projectile"),
                Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: animation_indices.first,
                    },
                ),
                animation_indices,
                AnimationTimer {
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                CastWeapon(fireball),
                Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.0)
                    .with_rotation(towards_quaternion),
                Direction(direction.extend(0.)),
                PlayerProjectile,
            ))
            .observe(on_fireball_hit);

        commands.spawn((
            SamplePlayer::new(asset_server.load("audio/sound_effects/fireball_whoosh.wav")),
            SfxPool,
        ));
    }

    Ok(())
}

fn on_fireball_hit(
    event: On<CollisionStart>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    weapon_q: Query<(&ExplosionRadius, &Damage), With<Fireball>>,
) -> Result {
    let projectile = event.collider1;
    let enemy = event.collider2;

    let (explosion_radius, dmg) = weapon_q.single()?;

    if let Ok((enemy_transform, enemy)) = enemy_q.get(enemy) {
        let dmg = dmg.0;

        spawn_visual_effect_at_hit_position(
            enemy_transform.translation,
            &mut commands,
            &asset_server,
            &mut texture_atlas_layout,
        )?;

        commands.trigger(EnemyDamageEvent {
            entity_hit: enemy,
            dmg,
            damage_type: DamageType::Fire,
        });

        for (other_enemy_transfor, other_enemy) in &enemy_q {
            if other_enemy_transfor == enemy_transform {
                //Skipp enemy hit
                continue;
            }
            let distance = enemy_transform
                .translation
                .truncate()
                .distance(other_enemy_transfor.translation.truncate());

            if distance < explosion_radius.0 {
                commands.trigger(EnemyDamageEvent {
                    entity_hit: other_enemy,
                    dmg,
                    damage_type: DamageType::Fire,
                });
            }
        }

        //Knockback
        commands.trigger(EnemyKnockbackEvent {
            entity_hit: enemy,
            projectile,
        });
    }
    commands.entity(projectile).despawn();

    Ok(())
}

fn spawn_visual_effect_at_hit_position(
    hit_position: Vec3,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> Result {
    let texture = asset_server.load("fx/fireball_hit.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(25, 30), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 7 };

    commands.spawn((
        Name::new("Fireball Impact Effect"),
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        animation_indices,
        AnimationTimer::once_from_fps(24),
        Transform::from_xyz(hit_position.x, hit_position.y, 0.0).with_scale(Vec3::splat(2.0)),
    ));
    commands.spawn((
        SamplePlayer::new(asset_server.load("audio/sound_effects/fireball_impact.wav")),
        SfxPool,
    ));

    Ok(())
}
