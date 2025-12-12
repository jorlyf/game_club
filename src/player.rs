use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, (move_player, animate_player_sprite));
  }
}

#[derive(Component)]
pub struct Player {
  pub name: String,
  pub speed: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
  Idle,
  Walking,
}

#[derive(Component, PartialEq, Clone, Copy)]
enum Direction {
  Down,
  Up,
  Left,
  Right,
}

const ANIMATION_FRAME_COUNT: usize = 4usize;

#[derive(Component)]
struct AnimationState {
  frame_index: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn get_animation_index_offset(direction: Direction) -> usize {
  const DOWN_OFFSET: usize = 0usize;
  const UP_OFFSET: usize = 1usize;
  const LEFT_OFFSET: usize = 2usize;
  const RIGHT_OFFSET: usize = 3usize;

  match direction {
    Direction::Down => DOWN_OFFSET,
    Direction::Up => UP_OFFSET,
    Direction::Left => LEFT_OFFSET,
    Direction::Right => RIGHT_OFFSET,
  }
}

pub fn spawn_player(
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
  position: Vec2,
) {
  let name = "jorlyf".to_string();
  let speed = 100f32;

  let texture_sheet = asset_server.load("player/sheet.png");

  let layout = TextureAtlasLayout::from_grid(
    UVec2::new(17, 27),
    ANIMATION_FRAME_COUNT as u32,
    4,
    Some(UVec2::splat(1)),
    None,
  );

  let texture_atlas_layout = texture_atlas_layouts.add(layout);

  commands.spawn((
    Player { name, speed },
    PlayerState::Idle,
    Direction::Down,
    AnimationState { frame_index: 0 },
    Transform {
      translation: position.extend(1.0),
      ..Default::default()
    },
    Sprite::from_atlas_image(
      texture_sheet,
      TextureAtlas {
        layout: texture_atlas_layout,
        index: get_animation_index_offset(Direction::Down),
      },
    ),
    AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
  ));
}

fn move_player(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  single: Single<(&mut Transform, &mut Direction, &mut PlayerState, &Player)>,
  time: Res<Time>,
) {
  let (mut transform, mut direction, mut state, player) = single.into_inner();

  let mut move_delta = Vec2::ZERO;

  if keyboard_input.pressed(KeyCode::KeyA) {
    move_delta.x -= 1.0;
  }

  if keyboard_input.pressed(KeyCode::KeyD) {
    move_delta.x += 1.0;
  }

  if keyboard_input.pressed(KeyCode::KeyW) {
    move_delta.y += 1.0;
  }

  if keyboard_input.pressed(KeyCode::KeyS) {
    move_delta.y -= 1.0;
  }

  if move_delta == Vec2::ZERO {
    *state = PlayerState::Idle;
    return;
  }

  *state = PlayerState::Walking;

  move_delta = move_delta.normalize() * player.speed * time.delta_secs();

  if move_delta.y < 0.0 {
    *direction = Direction::Down;
  } else if move_delta.y > 0.0 {
    *direction = Direction::Up;
  } else if move_delta.x < 0.0 {
    *direction = Direction::Left;
  } else if move_delta.x > 0.0 {
    *direction = Direction::Right;
  }

  transform.translation += move_delta.extend(0.0);
}

fn animate_player_sprite(
  time: Res<Time>,
  mut query: Query<
    (
      &mut AnimationTimer,
      &mut Sprite,
      &mut AnimationState,
      &PlayerState,
      &Direction,
    ),
    With<Player>,
  >,
) {
  for (mut timer, mut sprite, mut animation_state, player_state, direction) in &mut query {
    timer.tick(time.delta());

    let Some(atlas) = &mut sprite.texture_atlas else {
      continue;
    };

    match *player_state {
      PlayerState::Idle => {
        animation_state.frame_index = 0;
        let offset = get_animation_index_offset(*direction);
        atlas.index = animation_state.frame_index * ANIMATION_FRAME_COUNT + offset;
      }
      PlayerState::Walking => {
        if timer.just_finished() {
          animation_state.frame_index = if animation_state.frame_index == ANIMATION_FRAME_COUNT - 1
          {
            0
          } else {
            animation_state.frame_index + 1
          };

          let offset = get_animation_index_offset(*direction);
          atlas.index = animation_state.frame_index * ANIMATION_FRAME_COUNT + offset;
        }
      }
    }
  }
}
