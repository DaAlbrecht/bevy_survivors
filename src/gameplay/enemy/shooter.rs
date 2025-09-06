use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use crate::{
    gameplay::{
        enemy::{separation_force_calc, DamageCooldown, Enemy, KnockbackDirection, Speed, SPAWN_RADIUS}, player::{Direction, Player}, spells::{Cooldown, Damage, Halt, Knockback, Range, Root}, Health
    }, screens::Screen, AppSystems
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_shooter
            .run_if(on_timer(Duration::from_millis(2000)))
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update)        
    );
    app.add_systems(FixedUpdate, shooter_movement.run_if(in_state(Screen::Gameplay)));

    // app.add_systems(Update, (walker_movement).run_if(in_state(Screen::Gameplay)));
}

#[derive(Component)]
#[require(
    Health(10.),
    Speed(50.),
    KnockbackDirection(Direction(Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    })),
    Knockback(0.0),
    Damage(5.0), 
    DamageCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)),
    Cooldown(Timer::from_seconds(1.0,TimerMode::Once)),
    Range(100.0)
)]
pub(crate) struct Shooter;

fn spawn_shooter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut rng: GlobalEntropy<WyRand>,
    shooter_q: Query<&Shooter>,
) -> Result {
    let player_pos = player_query.single()?;

    let random_angle: f32 = rng.random_range(0.0..(2. * PI));
    let random_radius: f32 = rng.random_range(0.0..10.);
    let offset_x = (SPAWN_RADIUS + random_radius) * f32::sin(random_angle);
    let offset_y = (SPAWN_RADIUS + random_radius) * f32::cos(random_angle);

    let enemy_pos_x = player_pos.translation.x + offset_x;
    let enemy_pos_y = player_pos.translation.y + offset_y;

    let mut shooter_count = shooter_q.iter().count();
    shooter_count += 1;

    commands.spawn((
        Name::new(format!("Shooter {shooter_count}")),
        Enemy,
        Shooter,
        Sprite {
            image: asset_server.load("enemies/Shooter.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos_x, enemy_pos_y, 0.),
    ));

    Ok(())
}

fn shooter_movement(
    shooter_q: Query<(&mut Transform, &Speed, &Knockback), (With<Shooter>, Without<Root>, Without<Halt>)>,
    player_q: Query<&Transform, (With<Player>, Without<Shooter>)>, 
    time: Res<Time>,
) -> Result{
    let player_pos = player_q.single()?.translation.truncate(); 

    let shooter_positions = shooter_q
        .iter()
        .map(|t| t.0.translation.truncate())
        .collect::<Vec<Vec2>>();

    for (mut transform, speed, knockback) in shooter_q {
        let shoter_pos = transform.translation.truncate(); 
        if knockback.0 > 1.0 {
            //skip movement if enemy gets knockedback or is rooted
            continue;
        }     
    
        let direction = (player_pos - shoter_pos).normalize();

        let separation_force = separation_force_calc(&shooter_positions, shoter_pos, player_pos); 

        let movement = (direction + separation_force).normalize() * (speed.0 * time.delta_secs());
        transform.translation += movement.extend(0.0);
    }

    Ok(())
}
