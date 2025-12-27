use crate::PausableSystems;
use crate::gameplay::abilities::{
    Ability, AbilityAssets, AbilityCooldown, UseAbility, init_ability_assets, try_use_ability,
};
use crate::gameplay::player::Player;
use crate::screens::Screen;
use bevy::prelude::*;

#[derive(Component)]
#[require(
    Ability,
    AbilityAssets,
    AbilityCooldown(AbilityCooldown::ready(10.)),
    Name::new("Shield")
)]
#[derive(Reflect)]
pub(crate) struct Shield;

#[derive(Component)]
pub(crate) struct Shielded;

#[derive(Component)]
pub(crate) struct ShieldDuration(pub Timer);

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_use_shield);
    app.add_observer(shielded_added);
    app.add_observer(shielded_removed);
    app.add_observer(init_ability_assets::<Shield>("ui/icons/shield_spell.png"));
    app.add_systems(
        FixedUpdate,
        timer_tick_shielded
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn on_use_shield(
    trigger: On<UseAbility>,
    mut shield_q: Query<&mut AbilityCooldown, With<Shield>>,
    player_q: Query<Entity, With<Player>>,
    mut commands: Commands,
) -> Result {
    if !try_use_ability(trigger.ability_entity, &mut shield_q) {
        return Ok(());
    }

    let Ok(player) = player_q.single() else {
        return Ok(());
    };

    commands.entity(player).insert((
        Shielded,
        ShieldDuration(Timer::from_seconds(5.0, TimerMode::Once)),
    ));

    Ok(())
}

fn timer_tick_shielded(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShieldDuration), With<Shielded>>,
) -> Result {
    for (entity, mut shield_duration) in &mut query {
        shield_duration.0.tick(time.delta());

        if shield_duration.0.just_finished() {
            commands.entity(entity).remove::<Shielded>();
            commands.entity(entity).remove::<ShieldDuration>();
        }
    }

    Ok(())
}

fn shielded_added(
    event: On<Add, Shielded>,
    mut player_q: Query<&mut Sprite, With<Shielded>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(mut sprite) = player_q.get_mut(event.entity) else {
        return Ok(());
    };

    let shield_texture: Handle<Image> = asset_server.load("player_knight_shielded_.png");

    sprite.image = shield_texture;

    Ok(())
}

fn shielded_removed(
    event: On<Remove, Shielded>,
    mut player_q: Query<&mut Sprite, With<Shielded>>,
    asset_server: Res<AssetServer>,
) -> Result {
    let Ok(mut sprite) = player_q.get_mut(event.entity) else {
        return Ok(());
    };

    let shield_texture: Handle<Image> = asset_server.load("player_knight_.png");

    sprite.image = shield_texture;

    Ok(())
}
