use bevy::{platform::collections::HashSet, prelude::*, sprite::Anchor};
use bevy_seedling::sample::SamplePlayer;

use crate::{
    audio::SfxPool,
    gameplay::{
        enemy::{Enemy, EnemyDamageEvent},
        player::Player,
        spells::{Cooldown, Damage, Range, Spell, SpellType, UpgradeSpellEvent},
    },
    screens::Screen,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        cleanup_lightning_bolt.run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(spawn_lightning_bolt);
    app.add_observer(lightning_hit);
}

pub fn upgrade_lightning(
    _trigger: On<UpgradeSpellEvent>,
    mut lightning_q: Query<(&mut Cooldown, &mut Jumps), With<Lightning>>,
) -> Result {
    let (mut cooldown, mut jumps) = lightning_q.single_mut()?;

    let current_duration = cooldown.0.duration().as_secs_f32();
    let new_duration = (current_duration * 0.9).max(0.5); // Reduce by 10%, min 0.5s
    cooldown
        .0
        .set_duration(std::time::Duration::from_secs_f32(new_duration));
    info!("Lightning cooldown upgraded to: {}s", new_duration);

    jumps.0 += 1;
    info!("Lightning jumps upgraded to: {}", jumps.0);

    Ok(())
}

#[derive(Component)]
#[require(
    Spell,
    SpellType::Lightning,
    Damage(5.),
    Cooldown(Timer::from_seconds(3., TimerMode::Once,)),
    Jumps(3),
    Range(300.)
)]
#[derive(Reflect)]
pub(crate) struct Lightning;

#[derive(Event, Reflect)]
pub(crate) struct LightningAttackEvent;

#[derive(Component, Reflect)]
pub(crate) struct LightningVisualTimer(pub Timer);

#[derive(Event, Reflect)]
pub(crate) struct LightningHitEvent {
    pub enemy: Entity,
}

#[derive(Component, Reflect)]
pub(crate) struct Jumps(pub u32);

fn spawn_lightning_bolt(
    _trigger: On<LightningAttackEvent>,
    mut commands: Commands,
    player_q: Query<(&Transform, Entity), (With<Player>, Without<Enemy>)>,
    enemy_q: Query<(&Transform, Entity), (With<Enemy>, Without<Player>)>,
    lightning_q: Query<(&Jumps, &Range), With<Lightning>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok((player_pos, player_entity)) = player_q.single() else {
        return Ok(());
    };

    let (lightning_jumps, lightning_range) = lightning_q.single()?;

    let mut current_source_pos = player_pos;
    let mut current_source_entity: Option<Entity> = Some(player_entity);
    let mut visited: HashSet<Entity> = HashSet::new();

    for _ in 0..lightning_jumps.0 {
        // Reset for current jump
        let mut max_distance = lightning_range.0;
        let mut closest: Option<(Entity, &Transform)> = None;

        //get target
        for (enemy_pos, enemy) in &enemy_q {
            // dont traget the source
            if (Some(enemy)) == current_source_entity {
                continue;
            }

            // don't rehit enemies
            if visited.contains(&enemy) {
                continue;
            }

            let distance = current_source_pos
                .translation
                .truncate()
                .distance(enemy_pos.translation.truncate());

            if distance < max_distance {
                max_distance = distance;
                closest = Some((enemy, enemy_pos));
            }
        }

        // no enemy found => stop chaning
        let Some((enemy, enemy_pos)) = closest else {
            break;
        };

        // spawn visual + trigger damage
        let direction = (enemy_pos.translation - current_source_pos.translation).truncate();
        let length = direction.length();
        let angle = direction.y.atan2(direction.x);
        let anchor_point = current_source_pos.translation.truncate() + direction * 0.5;

        commands.spawn((
            Name::new("LightningBolt"),
            Sprite {
                image: asset_server.load("fx/lightning.png"),
                custom_size: Some(Vec2::new(length, 13.0)),
                ..default()
            },
            Anchor::CENTER,
            Transform {
                translation: anchor_point.extend(10.0),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            LightningVisualTimer(Timer::from_seconds(0.1, TimerMode::Once)),
        ));

        commands.spawn((
            SamplePlayer::new(asset_server.load("audio/sound_effects/lightning.ogg")),
            SfxPool,
        ));

        commands.trigger(LightningHitEvent { enemy });

        //update chain state
        visited.insert(enemy);
        current_source_pos = enemy_pos;
        current_source_entity = Some(enemy);
    }

    Ok(())
}

fn cleanup_lightning_bolt(
    mut commands: Commands,
    time: Res<Time>,
    mut lightning_q: Query<(Entity, &mut LightningVisualTimer)>,
) {
    for (entity, mut lightning_timer) in &mut lightning_q {
        lightning_timer.0.tick(time.delta());

        if lightning_timer.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn lightning_hit(
    trigger: On<LightningHitEvent>,
    mut commands: Commands,
    lightning_dmg: Query<&Damage, With<Lightning>>,
) -> Result {
    let enemy = trigger.enemy;
    let dmg = lightning_dmg.single()?.0;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy,
        dmg,
    });
    Ok(())
}
