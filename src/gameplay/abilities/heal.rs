use crate::gameplay::Health;
use crate::gameplay::abilities::{
    Ability, AbilityAssets, AbilityCooldown, UseAbility, init_ability_assets, try_use_ability,
};
use crate::gameplay::damage_numbers::{DamageMessage, DamageType};
use crate::gameplay::player::Player;
use bevy::prelude::*;

#[derive(Component)]
#[require(
    Ability,
    AbilityAssets,
    AbilityCooldown(AbilityCooldown::ready(10.)),
    Name::new("Heal")
)]
#[derive(Reflect)]
pub(crate) struct Heal;

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_use_heal);
    app.add_observer(init_ability_assets::<Heal>("ui/icons/heal_spell.png"));
}

fn on_use_heal(
    trigger: On<UseAbility>,
    mut heal_q: Query<&mut AbilityCooldown, With<Heal>>,
    mut player_q: Query<(&mut Health, &Transform), With<Player>>,
    mut damage_writer: MessageWriter<DamageMessage>,
) {
    if !try_use_ability::<Heal>(trigger.ability_entity, &mut heal_q) {
        return;
    }

    let Ok((mut health, transform)) = player_q.single_mut() else {
        return;
    };

    let heal_amount = 30.0;
    health.0 = (health.0 + heal_amount).min(100.0);

    damage_writer.write(DamageMessage {
        amount: heal_amount as i32,
        world_pos: transform.translation.truncate(),
        crit: false,
        damage_type: DamageType::Heal,
    });
}
