use bevy::{platform::collections::HashSet, prelude::*, sprite::Anchor};

use crate::{
    gameplay::{
        attacks::{Attack, Cooldown, Damage, Range, SpellType},
        enemy::{Enemy, EnemyDamageEvent},
        player::{Player, spawn_player},
    },
    screens::Screen,
};

pub(crate) struct LightningPlugin;

impl Plugin for LightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_lightning).after(spawn_player));

        app.add_systems(
            FixedUpdate,
            cleanup_lightning_bolt.run_if(in_state(Screen::Gameplay)),
        );
        app.add_observer(spawn_lightning_bolt);
        app.add_observer(lightning_hit);
    }
}

const LIGHTNING_BASE_COOLDOWN: f32 = 3.0;
const LIGHTNING_BASE_DMG: f32 = 5.0;
const LIGHTNING_BASE_JUMPS: i32 = 3;
const LIGHTNING_BASE_RANGE: f32 = 300.0;

#[derive(Component)]
pub(crate) struct Lightning;

#[derive(Event)]
pub(crate) struct LightningAttackEvent;

#[derive(Component)]
pub(crate) struct LightningVisualTimer(pub Timer);

#[derive(Event)]
pub(crate) struct LightningHitEvent {
    pub enemy: Entity,
    pub lightning_bolt: Entity,
}

#[derive(Component)]
pub(crate) struct Jumps(pub i32);

fn spawn_lightning(mut commands: Commands) {
    commands.spawn((
        Attack,
        Lightning,
        SpellType::Lightning,
        Cooldown(Timer::from_seconds(
            LIGHTNING_BASE_COOLDOWN,
            TimerMode::Once,
        )),
        Damage(LIGHTNING_BASE_DMG),
        Jumps(LIGHTNING_BASE_JUMPS),
        Range(LIGHTNING_BASE_RANGE),
    ));
}

fn spawn_lightning_bolt(
    _trigger: Trigger<LightningAttackEvent>,
    mut commands: Commands,
    player_q: Query<(&Transform, Entity), (With<Player>, Without<Enemy>)>,
    enemy_q: Query<(&Transform, Entity), (With<Enemy>, Without<Player>)>,
    lightning_q: Query<(&Damage, &Jumps, &Range), With<Lightning>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let (player_pos, player_entity) = player_q.single()?;
    let (lightning_dmg, lightning_jumps, lightning_range) = lightning_q.single()?;

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

        let lightning_bolt = commands
            .spawn((
                Sprite {
                    image: asset_server.load("Lightning.png"),
                    custom_size: Some(Vec2::new(length, 13.0)),
                    anchor: Anchor::Center,
                    ..default()
                },
                Transform {
                    translation: anchor_point.extend(1.0),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                Attack,
                SpellType::Lightning,
                Damage(lightning_dmg.0),
                LightningVisualTimer(Timer::from_seconds(0.1, TimerMode::Once)),
            ))
            .id();

        commands.trigger(LightningHitEvent {
            enemy: enemy,
            lightning_bolt,
        });

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

        if lightning_timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn lightning_hit(trigger: Trigger<LightningHitEvent>, mut commands: Commands) {
    let enemy_entity = trigger.enemy;
    let lightning_entity = trigger.lightning_bolt;

    commands.trigger(EnemyDamageEvent {
        entity_hit: enemy_entity,
        spell_entity: lightning_entity,
    });
}
