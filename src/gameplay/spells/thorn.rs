use bevy::{ecs::system::command, prelude::*};

use crate::{
    PLAYER_SIZE, SPELL_SIZE,
    gameplay::{
        enemy::{Enemy, Speed},
        player::{self, Direction, Player},
        spells::{
            CastSpell, Cooldown, Damage, Halt, Knockback, PlayerProjectile, ProjectileCount, Spell,
            SpellProjectiles, SpellType,
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

#[derive(Event)]
pub(crate) struct ThornAttackEvent;

#[derive(Event)]
pub(crate) struct ThornBaseSpawnEvent;

#[derive(Event)]
pub(crate) struct ThornHitEvent {
    pub enemy: Entity,
    pub projectile: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        thorn_range_keeper.run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(spawn_thorn_projectile);
}

fn spawn_thorn_projectile(
    _trigger: Trigger<ThornAttackEvent>,
    player_pos_q: Query<&Transform, With<Player>>,
    thorn_q: Query<(Entity), With<Thorn>>,
    enemy_pos_q: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    info!("shoot");
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
            ThornTip,
            PlayerProjectile,
        ));
    }
    Ok(())
}

fn thorn_range_keeper(
    mut thorn_q: Query<(&ProjectileCount, &mut Speed), With<Thorn>>,
    mut thorn_tip_q: Query<(Entity, &Transform), With<ThornTip>>,
    player_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
) -> Result {
    let (thorn_count, mut speed) = thorn_q.single_mut()?;
    let player_pos = player_q.single()?.translation.truncate();

    //Halt component?
    for (thron_tip, transform) in &mut thorn_tip_q {
        let distance = transform.translation.truncate().distance(player_pos);

        info!("Thorn_count: {}", thorn_count.0);
        info!("Thorn_speed: {}", speed.0);

        if distance >= thorn_count.0 * SPELL_SIZE {
            commands.entity(thron_tip).insert(Halt);
        }
    }

    Ok(())
}

fn thorn_base_spawner() {}
