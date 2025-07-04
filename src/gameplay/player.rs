use std::f32::consts::PI;

use bevy::{color::palettes::css, prelude::*};
use bevy_rand::{global::GlobalEntropy, prelude::WyRand};
use leafwing_input_manager::prelude::*;
use rand::Rng;

use super::enemy::{DamageCooldown, Health, Speed};
use super::healthbar::HealthBarMaterial;
use super::movement::{MovementController, apply_movement};
use crate::{AppSystem, screens::Screen};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);

        app.add_systems(OnEnter(Screen::Gameplay), show_player);

        app.add_systems(
            Update,
            (
                record_player_directional_input.in_set(AppSystem::RecordInput),
                player_shoot.after(apply_movement),
                update_player_timer,
                move_player_spell.after(player_shoot),
                update_health_bar,
            )
                .run_if(in_state(Screen::Gameplay)),
        );
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

        app.register_type::<XP>().register_type::<Level>();
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    input_manager: InputMap<PlayerAction>,
    movement_controller: MovementController,
}

#[derive(Component)]
pub struct PlayerSpell;

#[derive(Component)]
pub struct Direction(Vec3);

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    // Movement
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct XpCollectionRange(pub f32);

#[derive(Component, Reflect)]
pub struct XP(pub f32);

#[derive(Component, Reflect)]
pub struct Level(pub f32);

impl PlayerAction {
    pub const DIRECTIONS: [Self; 4] = [
        PlayerAction::Up,
        PlayerAction::Down,
        PlayerAction::Left,
        PlayerAction::Right,
    ];

    pub fn direction(self) -> Dir2 {
        match self {
            PlayerAction::Up => Dir2::Y,
            PlayerAction::Down => Dir2::NEG_Y,
            PlayerAction::Left => Dir2::NEG_X,
            PlayerAction::Right => Dir2::X,
        }
    }
}

impl PlayerBundle {
    fn default_input_map() -> InputMap<PlayerAction> {
        use PlayerAction::{Down, Left, Right, Up};
        let mut input_map = InputMap::default();

        // Movement
        input_map.insert(Up, KeyCode::KeyW);
        input_map.insert(Up, GamepadButton::DPadUp);

        input_map.insert(Down, KeyCode::KeyS);
        input_map.insert(Down, GamepadButton::DPadDown);

        input_map.insert(Left, KeyCode::KeyA);
        input_map.insert(Left, GamepadButton::DPadLeft);

        input_map.insert(Right, KeyCode::KeyD);
        input_map.insert(Right, GamepadButton::DPadRight);

        input_map
    }
}

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
            PlayerBundle {
                player: Player,
                input_manager: PlayerBundle::default_input_map(),
                movement_controller: MovementController {
                    max_speed: 100.0,
                    ..default()
                },
            },
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

fn update_health_bar(
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
    player_q: Query<&Health, With<Player>>,
    healthbar_material_q: Query<&MeshMaterial2d<HealthBarMaterial>>,
) -> Result {
    let health = player_q.single()?;
    let per = health.0 / 100.;
    let handle = healthbar_material_q.single()?.clone_weak();
    let material = health_bar_materials.get_mut(&handle).unwrap();
    material.percent = per;

    Ok(())
}

fn record_player_directional_input(
    action_state: Single<&ActionState<PlayerAction>, With<Player>>,
    mut controller_q: Query<&mut MovementController, With<Player>>,
) -> Result {
    let mut intent = Vec2::ZERO;
    let mut controller = controller_q.single_mut()?;

    for input_direction in PlayerAction::DIRECTIONS {
        if action_state.pressed(&input_direction) {
            let direction = input_direction.direction();
            intent += *direction;
        }
    }
    let intent = intent.normalize_or_zero();

    controller.intent = intent;
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
        let direction = Vec3::new(f32::sin(random_angle), f32::cos(random_angle), 0.);

        commands.spawn((
            Sprite {
                image: asset_server.load("Bullet.png"),
                ..default()
            },
            Transform::from_xyz(player_pos.translation.x, player_pos.translation.y, 0.),
            PlayerSpell,
            Speed(600.),
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
