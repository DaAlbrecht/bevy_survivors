use bevy::{prelude::*, sprite::Text2dShadow, text::FontSmoothing};

use crate::{asset_tracking::LoadResource, screens::Screen};

#[derive(Message)]
pub struct DamageMessage {
    pub amount: i32,
    pub world_pos: Vec2,
    pub crit: bool,
}

#[derive(Component)]
struct DamageNumber {
    timer: Timer,
    start_pos: Vec3,
    float_distance: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<DamageAssets>();
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
        let color = if msg.crit {
            Color::srgb(1.0, 0.9, 0.2)
        } else {
            Color::WHITE
        };

        let font_size = if msg.crit { 24.0 } else { 12.0 };

        let base_pos = Vec3::new(msg.world_pos.x, msg.world_pos.y + 10.0, 20.0);
        let snapped = Vec3::new(base_pos.x.round(), base_pos.y.round(), base_pos.z);

        commands.spawn((
            Text2d::new(msg.amount.to_string()),
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
            Transform::from_translation(snapped),
            DamageNumber {
                timer: Timer::from_seconds(0.9, TimerMode::Once),
                start_pos: snapped,
                float_distance: 20.0,
            },
        ));
    }
}

fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut DamageNumber, &mut Transform, &mut TextColor), With<Text2d>>,
) {
    for (entity, mut dmg, mut transform, mut text_color) in &mut q {
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

        if dmg.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct DamageAssets {
    #[dependency]
    pub(crate) font: Handle<Font>,
}

impl FromWorld for DamageAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            font: assets.load("ui/compass.ttf"),
        }
    }
}
