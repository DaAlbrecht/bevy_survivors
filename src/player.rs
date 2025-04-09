use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{AppSet, enemy::Health, movement::MovementController};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            record_player_directional_input.in_set(AppSet::RecordInput),
        );
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    input_manager: InputManagerBundle<PlayerAction>,
    movement_controller: MovementController,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    // Movement
    Up,
    Down,
    Left,
    Right,
}

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

#[derive(Debug)]
pub struct SpawnPlayer {
    /// See [`MovementController::max_speed`].
    pub max_speed: f32,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let _ = world.run_system_cached_with(spawn_player, self);
    }
}

fn spawn_player(
    In(config): In<SpawnPlayer>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Name::new("Player"),
        Sprite::from_image(asset_server.load("Player.png")),
        Transform::from_xyz(50., 0., 0.),
        PlayerBundle {
            player: Player,
            input_manager: InputManagerBundle::with_map(PlayerBundle::default_input_map()),
            movement_controller: MovementController {
                max_speed: config.max_speed,
                ..default()
            },
        },
        Health(100.),
    ));
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
