use bevy_enhanced_input::prelude::*;
use std::f32::consts::PI;

use bevy::{color::palettes::css, prelude::*};
use bevy_enhanced_input::action::Action;
use bevy_enhanced_input::actions;
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use rand::Rng;

use super::enemy::{DamageCooldown, Health, Speed};
use super::healthbar::HealthBarMaterial;
use crate::{AppSystem, screens::Screen};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>();

        app.add_systems(Startup, spawn_player);

        app.add_systems(OnEnter(Screen::Gameplay), show_player);

        app.add_systems(
            Update,
            (
                move_player.in_set(AppSystem::RecordInput),
                player_shoot,
                update_player_timer,
            )
                .run_if(in_state(Screen::Gameplay)),
        );

        app.add_systems(
            FixedUpdate,
            (move_player_spell).run_if(in_state(Screen::Gameplay)),
        );

        app.add_observer(player_hit);

        app.register_type::<XP>().register_type::<Level>();
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerSpell;

#[derive(Event)]
pub struct PlayerHitEvent {
    pub dmg: f32,
}

#[derive(Component)]
pub struct Direction(pub Vec3);

#[derive(Component)]
pub struct XpCollectionRange(pub f32);

#[derive(Component, Reflect)]
pub struct XP(pub f32);

#[derive(Component, Reflect)]
pub struct Level(pub f32);

#[derive(Component, Reflect)]
pub struct Knockback(pub f32);

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Move;

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    commands
        .spawn((
            Name::new("Player"),
            Sprite::from_image(asset_server.load("Player.png")),
            Transform::from_xyz(50., 0., 0.),
            Player,
            actions!(Player[
                (
                    Action::<Move>::new(),
                    SmoothNudge::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick()
                    )),
                    Scale::splat(100.),
                ),
            ]),
            Health(100.),
            DamageCooldown(Timer::from_seconds(1.0, TimerMode::Once)),
            XpCollectionRange(150.0),
            XP(0.),
            Level(1.),
            Visibility::Hidden,
        ))
        .with_child((
            Mesh2d(mesh.add(Rectangle::new(32.0, 5.0))),
            MeshMaterial2d(health_bar_materials.add(HealthBarMaterial {
                foreground_color: css::GREEN.into(),
                background_color: css::RED.into(),
                percent: 1.,
            })),
            Transform::from_xyz(0.0, -25.0, 0.),
        ));
}

fn player_hit(
    trigger: Trigger<PlayerHitEvent>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    mut player_q: Query<&mut Health, With<Player>>,
    healthbar_material_q: Query<&MeshMaterial2d<HealthBarMaterial>>,
) -> Result {
    let mut health = player_q.single_mut()?;
    health.0 -= trigger.dmg;
    debug!("attacking player, player_health: {}", health.0);
    let per = health.0 / 100.;

    let handle = healthbar_material_q.single()?.clone_weak();
    let material = health_bar_materials.get_mut(&handle).unwrap();
    material.percent = per;

    Ok(())
}

fn move_player(
    move_action: Single<&Action<Move>>,
    mut player_transform_q: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) -> Result {
    let velocity = move_action.extend(0.0);
    let mut player_transform = player_transform_q.single_mut()?;
    player_transform.translation += velocity * time.delta_secs();

    Ok(())
}

fn player_shoot(
    mut player_cd_q: Query<&mut DamageCooldown, With<Player>>,
    player_pos_q: Query<&Transform, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: GlobalEntropy<WyRand>,
) -> Result {
    let player_pos = player_pos_q.single()?;
    let mut player_cd = player_cd_q.single_mut()?;
    let random_angle: f32 = rng.gen_range(0.0..(2. * PI));

    if player_cd.0.finished() {
        let direction = Vec3::new(f32::cos(random_angle), f32::sin(random_angle), 0.).normalize();

        commands.spawn((
            Name::new("Default Attack"),
            Sprite {
                image: asset_server.load("Bullet.png"),
                ..default()
            },
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
            PlayerSpell,
            Speed(600.),
            Knockback(1500.),
            Direction(direction),
        ));
        player_cd.0.reset();
    }

    Ok(())
}

fn update_player_timer(time: Res<Time>, mut cooldowns: Query<&mut DamageCooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }
}

fn move_player_spell(
    mut bullet_pos_q: Query<
        (&mut Transform, &Speed, &Direction),
        (With<PlayerSpell>, Without<Player>),
    >,
    time: Res<Time>,
) -> Result {
    for (mut bullet_pos, bullet_speed, bullet_direction) in &mut bullet_pos_q {
        let movement = bullet_direction.0 * bullet_speed.0 * time.delta_secs();
        bullet_pos.translation += movement;
    }

    Ok(())
}

fn show_player(mut visibility_q: Query<&mut Visibility, With<Player>>) -> Result {
    let mut visibility = visibility_q.single_mut()?;
    *visibility = Visibility::Visible;
    Ok(())
}
