use crate::{PausableSystems, screens::Screen};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{InputAction, Start};

pub mod dash;
pub mod heal;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((dash::plugin, heal::plugin));

    app.add_systems(
        FixedUpdate,
        (handle_timers,)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_observer(on_q_pressed);
    app.add_observer(on_e_pressed);
    app.add_observer(on_r_pressed);
}

#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct UseQAbility;

#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct UseEAbility;

#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct UseRAbility;

#[derive(Event)]
pub struct UseAbility {
    pub ability_entity: Entity,
}

fn on_q_pressed(
    _trigger: On<Start<UseQAbility>>,
    q_ability: Query<Entity, With<QAbility>>,
    mut commands: Commands,
) {
    if let Ok(entity) = q_ability.single() {
        commands.trigger(UseAbility {
            ability_entity: entity,
        });
    }
}

fn on_e_pressed(
    _trigger: On<Start<UseEAbility>>,
    e_ability: Query<Entity, With<EAbility>>,
    mut commands: Commands,
) {
    if let Ok(entity) = e_ability.single() {
        commands.trigger(UseAbility {
            ability_entity: entity,
        });
    }
}

fn on_r_pressed(
    _trigger: On<Start<UseRAbility>>,
    r_ability: Query<Entity, With<RAbility>>,
    mut commands: Commands,
) {
    if let Ok(entity) = r_ability.single() {
        commands.trigger(UseAbility {
            ability_entity: entity,
        });
    }
}

#[derive(Component)]
pub struct QAbility;

#[derive(Component)]
pub struct EAbility;

#[derive(Component)]
pub struct RAbility;

#[derive(Component, Default, Reflect)]
pub(crate) struct Ability;

#[derive(Component, Default, Reflect)]
pub(crate) struct AbilityCooldown(pub Timer);

impl AbilityCooldown {
    pub fn ready(seconds: f32) -> Timer {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        timer.tick(timer.duration());
        timer
    }
}

#[derive(Component, Reflect)]
pub(crate) struct AbilityDuration(pub Timer);

#[derive(Component, Clone, Default)]
pub struct AbilityAssets {
    pub icon: Handle<Image>,
}

/// Call this on ability init to load its assets
pub fn init_ability_assets<T: Component>(
    icon_path: &'static str,
) -> impl Fn(On<Add, T>, Query<&mut AbilityAssets, With<T>>, Res<AssetServer>) {
    move |_trigger: On<Add, T>,
          mut query: Query<&mut AbilityAssets, With<T>>,
          asset_server: Res<AssetServer>| {
        for mut assets in &mut query {
            assets.icon = asset_server.load(icon_path);
        }
    }
}

/// Try using an ability, checking its cooldown
pub fn try_use_ability<T: Component>(
    ability_entity: Entity,
    ability_query: &mut Query<&mut AbilityCooldown, With<T>>,
) -> bool {
    let Ok(mut cooldown) = ability_query.get_mut(ability_entity) else {
        return false; // Not this ability type
    };

    if !cooldown.0.is_finished() {
        return false; // On cooldown
    }

    cooldown.0.reset();
    true
}

fn handle_timers(
    time: Res<Time>,
    mut cooldowns: Query<&mut AbilityCooldown>,
    mut durations: Query<&mut AbilityDuration>,
) {
    for mut cooldown in &mut cooldowns {
        cooldown.0.tick(time.delta());
    }

    for mut duration in &mut durations {
        duration.0.tick(time.delta());
    }
}
