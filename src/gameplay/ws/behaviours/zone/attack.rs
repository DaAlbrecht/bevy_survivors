// TODO: BEAM
//
// let player_pos_2d = player_pos.translation.truncate();
// let enemy_pos_2d = enemy_pos.translation.truncate();
// let direction = (enemy_pos_2d - player_pos_2d).normalize();
// let distance = player_pos_2d.distance(enemy_pos_2d);
//
// // Position beam at midpoint between player and enemy
// let midpoint = player_pos_2d + direction * (distance / 2.0);
//
// // Calculate rotation to point at enemy
// let angle = direction.y.atan2(direction.x);
//
// // Create stretched sprite and collider
// let mut beam = commands.spawn((
//     Name::new("Zone Beam"),
//     CastWeapon(weapon),
//     Transform::from_xyz(midpoint.x, midpoint.y, 9.0)
//         .with_rotation(Quat::from_rotation_z(angle)),
//     super::ZoneBeam,
//     Collider::rectangle(distance, width.0),
//     CollisionEventsEnabled,
//     Sensor,
//     CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default]),
//     WeaponDuration(Timer::from_seconds(lifetime.0, TimerMode::Once)),
// ));

//TODO: CONE SHAPE
//
//
// let player_pos_2d = player_pos.translation.truncate();
//
// // Get player facing direction (default to right if no direction component)
// let direction = Vec2::X;
// let angle = direction.y.atan2(direction.x);
//
// // Create cone shape using triangle collider
// // Cone starts at player and expands outward
// let half_angle = angle_degrees.to_radians() / 2.0;
// let left_direction = Vec2::from_angle(angle + half_angle);
// let right_direction = Vec2::from_angle(angle - half_angle);
//
// // Three points of the cone: apex (player), left edge, right edge
// let apex = Vec2::ZERO;
// let left_point = left_direction * range;
// let right_point = right_direction * range;
