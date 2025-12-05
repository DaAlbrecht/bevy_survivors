use crate::gameplay::Health;
use crate::gameplay::abilities::{
    Ability, AbilityAssets, AbilityCooldown, UseAbility, init_ability_assets, try_use_ability,
};
use crate::gameplay::damage_numbers::{DamageMessage, DamageType};
use crate::gameplay::healthbar::HealthBarMaterial;
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
    healthbar_material_q: Query<&MeshMaterial2d<HealthBarMaterial>>,
    mut health_bar_materials: ResMut<Assets<HealthBarMaterial>>,
) -> Result {
    if !try_use_ability::<Heal>(trigger.ability_entity, &mut heal_q) {
        return Ok(());
    }

    let Ok((mut health, transform)) = player_q.single_mut() else {
        return Ok(());
    };

    let heal_amount = 30.0;
    health.0 = (health.0 + heal_amount).min(100.0);

    let per = health.0 / 100.;

    let handle = healthbar_material_q.single()?.clone();
    let material = health_bar_materials.get_mut(&handle).unwrap();
    material.percent = per;

    damage_writer.write(DamageMessage {
        amount: heal_amount as i32,
        world_pos: transform.translation.truncate(),
        crit: false,
        damage_type: DamageType::Heal,
    });

    Ok(())
}
