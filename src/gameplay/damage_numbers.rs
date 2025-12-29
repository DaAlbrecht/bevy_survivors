use bevy::{prelude::*, sprite::Text2dShadow, text::FontSmoothing};
use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{AssetStates, screens::Screen};
use rand::Rng;

#[derive(Copy, Clone, Reflect, Deserialize, Serialize, Debug)]
pub enum DamageType {
    Physical,
    Fire,
    Lightning,
    Ice,
    Earth,
    Heal,
    Bleed,
}

impl DamageType {
    pub fn to_icon_handle(self, assets: &DamageAssets) -> Option<Handle<Image>> {
        match self {
            DamageType::Fire => Some(assets.fire.clone()),
            DamageType::Lightning => Some(assets.lightning.clone()),
            DamageType::Ice => Some(assets.ice.clone()),
            DamageType::Physical | DamageType::Earth => None,
            DamageType::Heal => Some(assets.heart.clone()),
            DamageType::Bleed => Some(assets.blood.clone()),
        }
    }
}

#[derive(Message)]
pub struct DamageMessage {
    pub amount: i32,
    pub world_pos: Vec2,
    pub crit: bool,
    pub damage_type: DamageType,
}

#[derive(Component)]
struct DamageNumber {
    timer: Timer,
    start_pos: Vec3,
    float_distance: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AssetStates::AssetLoading).load_collection::<DamageAssets>(),
    );
    app.add_message::<DamageMessage>();

    app.add_systems(
        Update,
        (spawn_damage_numbers_from_messages, update_damage_numbers)
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn spawn_damage_numbers_from_messages(
    mut commands: Commands,
    mut reader: MessageReader<DamageMessage>,
    damage_assets: Res<DamageAssets>,
) {
    for msg in reader.read() {
        //TODO: Move this into a function for extensibility or better implement over DamageType
        let is_heal = matches!(msg.damage_type, DamageType::Heal);

        let color = if is_heal {
            Color::srgb(0.2, 1.0, 0.3)
        } else if msg.crit {
            Color::srgb(1.0, 0.9, 0.2)
        } else {
            Color::WHITE
        };

        let font_size = if msg.crit { 24.0 } else { 12.0 };
        let mut rng = rand::rng();
        let mut base_pos = Vec3::new(msg.world_pos.x, msg.world_pos.y + 10.0, 20.0);

        base_pos.x += rng.random_range(-5.0..5.0);
        let snapped = Vec3::new(base_pos.x.round(), base_pos.y.round(), base_pos.z);

        let parent = commands
            .spawn((
                DamageNumber {
                    timer: Timer::from_seconds(0.9, TimerMode::Once),
                    start_pos: snapped,
                    float_distance: 20.0,
                },
                Transform::from_translation(snapped),
                Text2d::new(format_damage_number(msg.amount)),
                TextFont {
                    font: damage_assets.font.clone(),
                    font_size,
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Center),
                TextColor(color),
                Text2dShadow {
                    color: Color::srgb(0.0, 0.0, 0.0),
                    offset: Vec2::new(1.0, -1.0),
                },
            ))
            .id();

        if let Some(icon_handle) = msg.damage_type.to_icon_handle(&damage_assets) {
            commands.entity(parent).with_children(|parent| {
                parent.spawn((
                    Sprite {
                        image: icon_handle,
                        custom_size: Some(Vec2::splat(font_size)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(10.0, 0.0, -0.1)),
                ));
            });
        }
    }
}

fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(
        Entity,
        &mut DamageNumber,
        &mut Transform,
        &mut TextColor,
        Option<&Children>,
    )>,
    mut sprite_q: Query<Option<&mut Sprite>>,
) {
    for (entity, mut dmg, mut transform, mut text_color, children) in &mut q {
        dmg.timer.tick(time.delta());

        let duration = dmg.timer.duration().as_secs_f32();
        let t = (dmg.timer.elapsed_secs() / duration).clamp(0.0, 1.0);

        let y_offset = dmg.float_distance * t;
        transform.translation.y = dmg.start_pos.y + y_offset;

        let scale = 1.0 + 0.3 * (1.0 - t);
        transform.scale = Vec3::splat(scale);

        let fade = (1.0 - t).powf(1.8);

        let mut c = text_color.0;
        c.set_alpha(fade);
        text_color.0 = c;

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(Some(mut sprite)) = sprite_q.get_mut(child) {
                    let mut color = sprite.color;
                    color.set_alpha(fade);
                    sprite.color = color;
                }
            }
        }

        if dmg.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn format_damage_number(amount: i32) -> String {
    let sign = if amount.is_negative() { "-" } else { "" };
    let n = amount.unsigned_abs();

    if n >= 1_000_000 {
        format!("{sign}{:.1}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{sign}{:.1}k", n / 1_000)
    } else {
        format!("{sign}{n}")
    }
}

#[derive(AssetCollection, Resource)]
pub(crate) struct DamageAssets {
    #[asset(path = "ui/compass.ttf")]
    pub(crate) font: Handle<Font>,
    #[asset(path = "ui/icons/tag_life.png")]
    pub(crate) heart: Handle<Image>,
    #[asset(path = "ui/icons/tag_fire.png")]
    pub(crate) fire: Handle<Image>,
    #[asset(path = "ui/icons/tag_lightning.png")]
    pub(crate) lightning: Handle<Image>,
    #[asset(path = "ui/icons/tag_ice.png")]
    pub(crate) ice: Handle<Image>,
    #[asset(path = "ui/icons/tag_blood.png")]
    pub(crate) blood: Handle<Image>,
}
