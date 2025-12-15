use bevy::prelude::*;

pub struct SnakeGamePlugin;

impl Plugin for SnakeGamePlugin {
  fn build(&self, app: &mut App) {
    // app.add_systems(Update, spawn_snake);
  }
}

const WIDTH: u32 = 13;
const HEIGHT: u32 = 13;

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeSegment;

pub fn start() {}

fn spawn_snake(mut commands: Commands) {
  commands.spawn((
    SnakeHead,
    Sprite::from_color(Color::WHITE, Vec2::new(50.0, 50.0)),
    Transform {
      translation: Vec3::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, 0.0),
      ..default()
    },
  ));
}

fn snake_movement(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut snake: Single<&mut Transform, With<SnakeHead>>,
) {
  if keyboard_input.pressed(KeyCode::ArrowLeft) {
    snake.translation.x -= 1.0;
  }
  if keyboard_input.pressed(KeyCode::ArrowRight) {
    snake.translation.x += 1.0;
  }
  if keyboard_input.pressed(KeyCode::ArrowDown) {
    snake.translation.y -= 1.0;
  }
  if keyboard_input.pressed(KeyCode::ArrowUp) {
    snake.translation.y += 1.0;
  }
}
