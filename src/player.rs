use avian2d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, sprite::Anchor};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app.add_message::<SpawnPlayerMessage>();
    app.add_message::<DespawnPlayerMessage>();

    app.add_systems(Update, (spawn_player_system, despawn_player_system));

    app.add_systems(Update, (move_player_system, animate_player_sprite_system));
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

#[derive(Message)]
pub struct SpawnPlayerMessage {
  pub position: Vec2,
}

#[derive(Message)]
pub struct DespawnPlayerMessage;

fn spawn_player_system(
  mut commands: Commands,
  mut spawn_player_messages: MessageReader<SpawnPlayerMessage>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  asset_server: Res<AssetServer>,
) {
  for message in spawn_player_messages.read() {
    let position = message.position;
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
      Name::new("Player"),
      Player { name, speed },
      PlayerState::Idle,
      Direction::Down,
      AnimationState { frame_index: 0 },
      Transform {
        translation: position.extend(1.0),
        ..Default::default()
      },
      RigidBody::Kinematic,
      Collider::circle(4.0),
      Sprite::from_atlas_image(
        texture_sheet,
        TextureAtlas {
          layout: texture_atlas_layout,
          index: get_animation_index_offset(Direction::Down),
        },
      ),
      Anchor::BOTTOM_CENTER,
      AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
  }
}

fn despawn_player_system(
  mut commands: Commands,
  despawn_player_messages: MessageReader<DespawnPlayerMessage>,
  player_query: Query<Entity, With<Player>>,
) {
  if despawn_player_messages.is_empty() {
    return;
  }

  let Ok(player_entity) = player_query.single() else {
    return;
  };

  commands.entity(player_entity).despawn();
}

fn move_player_system(
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

fn animate_player_sprite_system(
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
