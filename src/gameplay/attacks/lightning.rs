use bevy::{prelude::*, sprite::Anchor};

use crate::{
    gameplay::{
        attacks::{Attack, Cooldown, Damage, SpellType},
        enemy::Enemy,
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
    }
}

const LIGHTNING_BASE_COOLDOWN: f32 = 1.0;
const LIGHTNING_BASE_DMG: f32 = 5.0;
const LIGHTNING_BASE_JUMPS: i32 = 3;

#[derive(Component)]
pub(crate) struct Lightning;

#[derive(Event)]
pub(crate) struct LightningAttackEvent;

#[derive(Component)]
pub(crate) struct LightningVisualTimer(pub Timer);

#[derive(Event)]
pub(crate) struct LightningHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

#[derive(Component)]
pub(crate) struct Jumps(pub i32);

fn spawn_lightning(mut commands: Commands, player_q: Query<Entity, With<Player>>) -> Result {
    let player = player_q.single()?;

    let lighting = commands.spawn((
        Attack,
        Lightning,
        SpellType::Lightning,
        Cooldown(Timer::from_seconds(
            LIGHTNING_BASE_COOLDOWN,
            TimerMode::Once,
        )),
        Damage(LIGHTNING_BASE_DMG),
        Jumps(LIGHTNING_BASE_JUMPS),
    ));

    Ok(())
}

fn spawn_lightning_bolt(
    _trigger: Trigger<LightningAttackEvent>,
    mut commands: Commands,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemy_q: Query<(&Transform, Entity), (With<Enemy>, Without<Player>)>,
    config_q: Query<(&Damage, &Jumps), With<Lightning>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let player_pos = player_q.single()?;
    let (config_dmg, config_jumps) = config_q.single()?;

    let mut min_distance = f32::MAX;
    let mut closest_enemy: Option<Entity> = None;

    //get target
    for (enemy_pos, enemy) in &enemy_q {
        let distance = player_pos
            .translation
            .truncate()
            .distance(enemy_pos.translation.truncate());

        if distance < min_distance {
            min_distance = distance;
            closest_enemy = Some(enemy);
        }
    }

    if let Some(enemy) = closest_enemy {
        let (enemy_pos, _) = enemy_q.get(enemy)?;
        let direction = (enemy_pos.translation - player_pos.translation).truncate();
        let length = direction.length();
        let angle = direction.y.atan2(direction.x);
        let anchor_point = player_pos.translation.truncate() + direction * 0.5;

        commands.spawn((
            Sprite {
                image: asset_server.load("Lightning.png"),
                custom_size: Some(Vec2::new(length, 13.0)),
                anchor: Anchor::Center,
                ..default()
            },
            Transform {
                translation: anchor_point.extend(0.0),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            LightningVisualTimer(Timer::from_seconds(0.1, TimerMode::Once)),
        ));
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
