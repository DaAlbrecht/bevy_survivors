use crate::audio::SfxPool;
use crate::gameplay::damage_numbers::DamageType;
use crate::gameplay::player::Direction;
use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::gameplay::weapons::{UpgradeWeaponEvent, WeaponAttackEvent};
use crate::gameplay::{
    Speed,
    enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
    player::Player,
    weapons::{
        CastWeapon, Cooldown, Damage, ExplosionRadius, Knockback, PlayerProjectile, Weapon,
        WeaponType,
    },
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

#[derive(Component)]
#[require(
    Weapon,
    WeaponType::Icelance,
    Cooldown(Timer::from_seconds(0.5, TimerMode::Once)),
    Speed(400.),
    Knockback(100.),
    Damage(1.),
    ExplosionRadius(100.),
    Name::new("Icelance")
)]
#[derive(Reflect)]
pub(crate) struct Icelance;

#[derive(Event, Reflect)]
pub(crate) struct IcelanceAttackEvent;

// pub(crate) fn plugin(app: &mut App) {}

pub fn upgrade_icelance(
    _trigger: On<UpgradeWeaponEvent>,
    mut icelance_q: Query<&mut Damage, With<Icelance>>,
) -> Result {
    let mut damage = icelance_q.single_mut()?;
    damage.0 += 5.0;
    info!("Icelance damage upgraded to: {}", damage.0);

    Ok(())
}

pub fn spawn_icelance_projectile(
    _trigger: On<WeaponAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    icelance: Query<Entity, With<Icelance>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(player_pos) = player_q.single() else {
        return Ok(());
    };

    let icelance = icelance.single()?;

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

    let texture = asset_server.load("fx/icelance.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(8, 36), 6, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 5 };

    if let Some(enemy_pos) = closest_enemy {
        let spawn_height = 350.0;
        let spawn_position = Vec3::new(
            enemy_pos.translation.x,
            enemy_pos.translation.y + spawn_height,
            10.0,
        );

        commands
            .spawn((
                Name::new("icelance projectile"),
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
                CastWeapon(icelance),
                Transform::from_translation(spawn_position),
                Direction(Vec3::new(0.0, -1.0, 0.0)),
                PlayerProjectile,
            ))
            .observe(on_icelance_hit);
    }

    Ok(())
}

fn on_icelance_hit(
    event: On<CollisionStart>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    weapon_q: Query<(&ExplosionRadius, &Damage), With<Icelance>>,
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
            damage_type: DamageType::Ice,
        });

        for (other_enemy_transfor, other_enemy) in &enemy_q {
            if other_enemy_transfor == enemy_transform {
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
                    damage_type: DamageType::Ice,
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
    let texture = asset_server.load("fx/icelance_hit.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(31, 23), 4, 1, None, None);
    let texture_atlas_layout = texture_atlas_layout.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands.spawn((
        Name::new("Icelance Impact Effect"),
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        animation_indices,
        AnimationTimer::once_from_fps(24),
        Transform::from_xyz(hit_position.x, hit_position.y, 10.0).with_scale(Vec3::splat(2.0)),
    ));
    commands.spawn((
        SamplePlayer::new(asset_server.load("audio/sound_effects/ice_hit.wav")),
        SfxPool,
    ));

    Ok(())
}
