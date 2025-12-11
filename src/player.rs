use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, player_movement_system);
  }
}

#[derive(Component)]
pub struct Player {
  pub name: String,
  pub speed: f32,
}

pub fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>, position: Vec2) {
  let name = "jorlyf".to_string();
  let speed = 300f32;

  commands
    .spawn(Sprite::from_image(
      asset_server.load("player/player_sprite.png"),
    ))
    .insert(Player { name, speed })
    .insert(Transform {
      translation: position.extend(1.0),
      ..Default::default()
    });
}

fn player_movement_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  player_query: Single<(&mut Transform, &Player)>,
  time: Res<Time>,
) {
  let mut direction = Vec2::ZERO;

  let (mut transform, player) = player_query.into_inner();

  if keyboard_input.pressed(KeyCode::KeyA) {
    direction.x -= 1.0;
  }

  if keyboard_input.pressed(KeyCode::KeyD) {
    direction.x += 1.0;
  }

  if keyboard_input.pressed(KeyCode::KeyW) {
    direction.y += 1.0;
  }

  if keyboard_input.pressed(KeyCode::KeyS) {
    direction.y -= 1.0;
  }

  if direction != Vec2::ZERO {
    let delta = direction.normalize() * player.speed * time.delta_secs();
    transform.translation += delta.extend(0.0);
  }
}
