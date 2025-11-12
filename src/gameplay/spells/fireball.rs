use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;

use crate::audio::SfxPool;
use crate::gameplay::movement::{
    MovementController, PhysicalTranslation, PreviousPhysicalTranslation,
};
use crate::gameplay::simple_animation::{AnimationIndices, AnimationTimer};
use crate::gameplay::spells::UpgradeSpellEvent;
use crate::gameplay::{
    Speed,
    enemy::{Enemy, EnemyDamageEvent, EnemyKnockbackEvent},
    player::Player,
    spells::{
        CastSpell, Cooldown, Damage, ExplosionRadius, Knockback, PlayerProjectile, Spell, SpellType,
    },
};

#[derive(Component)]
#[require(
    Spell,
    SpellType::Fireball,
    Cooldown(Timer::from_seconds(5., TimerMode::Once)),
    Speed(600.),
    Knockback(100.),
    Damage(5.),
    ExplosionRadius(100.),
    Name::new("Fireball")
)]
#[derive(Reflect)]
pub(crate) struct Fireball;

#[derive(Event, Reflect)]
pub(crate) struct FireballAttackEvent;

#[derive(Event, Reflect)]
pub(crate) struct FireballHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(spawn_fireball_projectile);
    app.add_observer(fireball_hit);
    app.add_observer(upgrade_fireball);
}

fn upgrade_fireball(
    _trigger: On<UpgradeSpellEvent>,
    mut fireball_q: Query<&mut Damage, With<Fireball>>,
) -> Result {
    let mut damage = fireball_q.single_mut()?;
    damage.0 += 5.0;
    info!("Fireball damage upgraded to: {}", damage.0);

    Ok(())
}

fn spawn_fireball_projectile(
    _trigger: On<FireballAttackEvent>,
    player_q: Query<&Transform, With<Player>>,
    fireball: Query<Entity, With<Fireball>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player_q.single()?;
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

        commands.spawn((
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
            MovementController {
                velocity: direction.extend(0.),
                speed: 400.,
                mass: 80.0,
                ..default()
            },
            CastSpell(fireball),
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.)
                .with_rotation(towards_quaternion),
            PhysicalTranslation(Vec3::new(
                player_pos.translation.x,
                player_pos.translation.y,
                0.,
            )),
            PreviousPhysicalTranslation(Vec3::new(
                player_pos.translation.x,
                player_pos.translation.y,
                0.,
            )),
            PlayerProjectile,
        ));
        commands.spawn((
            SamplePlayer::new(asset_server.load("audio/sound_effects/fireball_whoosh.wav")),
            SfxPool,
        ));
    }

    Ok(())
}

fn fireball_hit(
    trigger: On<FireballHitEvent>,
    enemy_q: Query<(&Transform, Entity), With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layout: ResMut<Assets<TextureAtlasLayout>>,
    explosion_radius: Query<(&ExplosionRadius, &Damage), With<Fireball>>,
) -> Result {
    let enemy_entity = trigger.enemy;
    let spell_entity = trigger.projectile;
    let (explosion_radius, dmg) = explosion_radius.single()?;

    let dmg = dmg.0;

    //Spawn impact effect
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
        Transform::from_xyz(
            enemy_q.get(enemy_entity)?.0.translation.x,
            enemy_q.get(enemy_entity)?.0.translation.y,
            0.1,
        )
        .with_scale(Vec3::splat(2.0)),
    ));
    commands.spawn((
        SamplePlayer::new(asset_server.load("audio/sound_effects/fireball_impact.wav")),
        SfxPool,
    ));

    //Deal damage
    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy_entity,
        dmg,
    });

    //Deal damage to all enemys in explosion radius
    if let Ok((enemy_pos, _)) = enemy_q.get(enemy_entity) {
        for (other_enemy_pos, other_enemy) in &enemy_q {
            if other_enemy_pos == enemy_pos {
                //Skipp enemy hit
                continue;
            }
            let distance = enemy_pos
                .translation
                .truncate()
                .distance(other_enemy_pos.translation.truncate());

            if distance < explosion_radius.0 {
                commands.trigger(EnemyDamageEvent {
                    entity_hit: other_enemy,
                    dmg,
                });
            }
        }
    }

    //Knockback
    commands.trigger(EnemyKnockbackEvent {
        entity_hit: enemy_entity,
        spell_entity,
    });

    commands.entity(spell_entity).despawn();

    Ok(())
}
