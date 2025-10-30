use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(add_bleed_visual);
    app.add_observer(remove_bleed_visual);
}

//Component to give to the one apply the dot
#[derive(Component, Clone, Reflect)]
pub(crate) struct DoT {
    pub duration: Timer,
    pub tick: Timer,
    pub dmg_per_tick: f32,
}

#[derive(Component, Reflect)]
pub(crate) struct Bleed {
    pub duration: Timer,
    pub tick: Timer,
    pub dmg_per_tick: f32,
}

#[derive(Component, Reflect)]
pub(crate) struct BleedVisual;

fn add_bleed_visual(
    trigger: On<Insert, Bleed>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let target = trigger.entity;

    let bleed_visual = commands
        .spawn((
            Sprite {
                image: asset_server.load("blood.png"),
                ..default()
            },
            BleedVisual,
        ))
        .id();

    commands.entity(target).add_child(bleed_visual);
}

fn remove_bleed_visual(
    trigger: On<Remove, Bleed>,
    visual_q: Query<Entity, With<BleedVisual>>,
    children_q: Query<&Children>,
    mut commands: Commands,
) {
    let target = trigger.entity;

    let Ok(children) = children_q.get(target) else {
        return;
    };

    for &child in children {
        if visual_q.get(child).is_ok() {
            commands.entity(child).try_despawn();
        }
    }
}
