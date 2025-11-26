use crate::gameplay::abilities::{
    Ability, AbilityAssets, AbilityCooldown, UseAbility, init_ability_assets, try_use_ability,
};
use crate::gameplay::character_controller::CharacterController;
use crate::gameplay::player::Player;
use crate::{GameplaySystems, PausableSystems};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

#[derive(Component)]
#[require(
    Ability,
    AbilityAssets,
    AbilityCooldown(AbilityCooldown::ready(5.)),
    DashVelocity,
    Name::new("Dash")
)]
#[derive(Reflect)]
pub(crate) struct Dash;

#[derive(Component, Default, Reflect)]
pub(crate) struct DashVelocity(pub Vec2);

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_use_dash);
    app.add_observer(init_ability_assets::<Dash>("ui/icons/dash_spell.png"));
    app.add_systems(
        FixedUpdate,
        (decay_dash_velocity, apply_dash_to_controller)
            .chain()
            .in_set(PausableSystems)
            .in_set(GameplaySystems::MovementModify),
    );
}

fn on_use_dash(
    trigger: On<UseAbility>,
    mut cooldown_q: Query<&mut AbilityCooldown, With<Dash>>,
    mut velocity_q: Query<&mut DashVelocity, With<Dash>>,
    player_q: Query<&LinearVelocity, With<Player>>,
) {
    if !try_use_ability::<Dash>(trigger.ability_entity, &mut cooldown_q) {
        return;
    }

    let Ok(mut dash_velocity) = velocity_q.get_mut(trigger.ability_entity) else {
        return;
    };

    let Ok(linear_velocity) = player_q.single() else {
        return;
    };

    let current_velocity = linear_velocity.0;
    let dash_direction = current_velocity.normalize_or_zero();

    if dash_direction != Vec2::ZERO {
        dash_velocity.0 = dash_direction * 500.0;
    } else {
        dash_velocity.0 = Vec2::Y * 500.0;
    }
}

fn decay_dash_velocity(time: Res<Time>, mut dash_q: Query<&mut DashVelocity, With<Dash>>) {
    for mut dash_velocity in &mut dash_q {
        dash_velocity.0 *= 0.1_f32.powf(time.delta_secs());

        if dash_velocity.0.length_squared() < 1.0 {
            dash_velocity.0 = Vec2::ZERO;
        }
    }
}

fn apply_dash_to_controller(
    dash_q: Query<&DashVelocity, With<Dash>>,
    mut player_q: Query<&mut CharacterController, With<Player>>,
) {
    let total_dash_velocity: Vec2 = dash_q.iter().map(|v| v.0).sum();

    for mut controller in &mut player_q {
        controller.ability_velocity = total_dash_velocity;
    }
}
