use bevy::prelude::*;

use crate::enemy::EnemyDeathEvent;

pub struct ExperiencePlugin;
impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_xp_gem);
    }
}

#[derive(Component)]
pub struct XpGem;

fn spawn_xp_gem(
    trigger: Trigger<EnemyDeathEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Result {
    let enemy_pos = trigger.0.translation;

    commands.spawn((
        Sprite {
            image: asset_server.load("XP_GEM.png"),
            ..default()
        },
        Transform::from_xyz(enemy_pos.x, enemy_pos.y, 0.),
        XpGem,
    ));

    Ok(())
}

fn collect_xp_gem() {}
