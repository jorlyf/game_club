use bevy::prelude::*;

use crate::games::{CurrentGameState, GameType};

pub struct SnakeGamePlugin;

impl Plugin for SnakeGamePlugin {
  fn build(&self, app: &mut App) {
    app.init_state::<SnakeGameState>();

    app.init_resource::<DirectionAccumulator>();
    app.init_resource::<GameTimer>();

    app
      .add_systems(Update, start.run_if(switched_to_game))
      .add_systems(
        PreUpdate,
        wait_for_input.run_if(in_state(SnakeGameState::WaitPlayer)),
      )
      .add_systems(
        Update,
        (
          accumulate_input_system.run_if(in_state(SnakeGameState::Playing)),
          move_system.run_if(in_state(SnakeGameState::Playing)),
        )
          .chain(),
      );
  }
}

fn switched_to_game(config: Res<CurrentGameState>) -> bool {
  config.is_changed() && config.current_game == Some(GameType::Snake)
}

const ARENA_WIDTH: u32 = 13;
const ARENA_HEIGHT: u32 = 13;
const ARENA_CELL_SIZE: u32 = 7;
const ARENA_CELL_GAP: u32 = 1;
const ARENA_PIXEL_WIDTH: u32 = ARENA_WIDTH * ARENA_CELL_SIZE + (ARENA_WIDTH - 1) * ARENA_CELL_GAP;
const ARENA_PIXEL_HEIGHT: u32 =
  ARENA_HEIGHT * ARENA_CELL_SIZE + (ARENA_HEIGHT - 1) * ARENA_CELL_GAP;

const IMAGE_WIDTH: u32 = 252;
const IMAGE_HEIGHT: u32 = 128;

const STEP_TIME_SECONDS: f32 = 0.350;

const START_SNAKE_LENGTH: u32 = 2;

#[derive(Component, Clone, PartialEq, Eq)]
struct GridPosition {
  x: u32,
  y: u32,
}

impl GridPosition {
  fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }

  fn increment_x(&mut self) {
    if self.x == ARENA_WIDTH - 1 {
      self.x = 0;
    } else {
      self.x += 1;
    }
  }
  fn decrement_x(&mut self) {
    if self.x == 0 {
      self.x = ARENA_WIDTH - 1;
    } else {
      self.x -= 1;
    }
  }

  fn increment_y(&mut self) {
    if self.y == ARENA_HEIGHT - 1 {
      self.y = 0;
    } else {
      self.y += 1;
    }
  }
  fn decrement_y(&mut self) {
    if self.y == 0 {
      self.y = ARENA_HEIGHT - 1;
    } else {
      self.y -= 1;
    }
  }
}

#[derive(Component, Clone, Copy, Deref, DerefMut, PartialEq, Eq, Hash)]
struct SnakeId(u32);

impl SnakeId {
  fn new(id: u32) -> Self {
    Self(id)
  }
}

#[derive(Component)]
struct SnakeHead {
  direction: SnakeDirection,
}

#[derive(Component)]
struct SnakeSegment {
  prev: Option<Entity>,
}

impl Default for SnakeHead {
  fn default() -> Self {
    Self {
      direction: SnakeDirection::Right,
    }
  }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq)]
enum SnakeDirection {
  #[default]
  Down,
  Up,
  Left,
  Right,
}

impl SnakeDirection {
  fn get_opposite(&self) -> Self {
    match self {
      SnakeDirection::Down => SnakeDirection::Up,
      SnakeDirection::Up => SnakeDirection::Down,
      SnakeDirection::Left => SnakeDirection::Right,
      SnakeDirection::Right => SnakeDirection::Left,
    }
  }
}

#[derive(States, Debug, Clone, Hash, Eq, PartialEq, Default)]
enum SnakeGameState {
  #[default]
  WaitPlayer,
  Playing,
  GameOver,
  Win,
  ExitModal,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct DirectionAccumulator(SnakeDirection);

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

impl Default for GameTimer {
  fn default() -> Self {
    GameTimer(Timer::from_seconds(STEP_TIME_SECONDS, TimerMode::Repeating))
  }
}

fn start(
  mut commands: Commands,
  mut next_state: ResMut<NextState<SnakeGameState>>,
  asset_server: Res<AssetServer>,
) {
  spawn_snake(&mut commands);

  let (width, height) = spawn_background(&mut commands, asset_server);

  let mut projection = OrthographicProjection::default_2d();
  projection.scaling_mode = bevy::camera::ScalingMode::Fixed {
    width: width as f32,
    height: height as f32,
  };

  spawn_camera(&mut commands, &projection);

  next_state.set(SnakeGameState::WaitPlayer);
}

fn spawn_camera(commands: &mut Commands, projection: &OrthographicProjection) {
  commands.spawn((
    Camera2d,
    Camera {
      order: 1,
      ..Default::default()
    },
    Projection::Orthographic(projection.clone()),
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
  ));
}

fn spawn_snake(commands: &mut Commands) {
  let head = SnakeHead::default();
  let head_position = GridPosition::new(ARENA_WIDTH / 2, ARENA_HEIGHT / 2);

  let head_entity = commands.spawn((
    Name::new("SnakeHead"),
    SnakeId::new(1),
    head,
    SnakeSegment { prev: None },
    head_position.clone(),
    Sprite::from_color(
      Color::WHITE,
      Vec2::new(ARENA_CELL_SIZE as f32 / 1.5, ARENA_CELL_SIZE as f32 / 1.5),
    ),
    Transform { ..default() },
  ));

  let mut segment_next_position = head_position;
  let mut prev_segment = Some(head_entity.id());

  for _ in 0..START_SNAKE_LENGTH - 1 {
    segment_next_position.increment_x();
    prev_segment = Some(spawn_snake_segment(
      commands,
      segment_next_position.clone(),
      &prev_segment,
    ));
  }
}

fn spawn_snake_segment(
  commands: &mut Commands,
  position: GridPosition,
  prev_segment: &Option<Entity>,
) -> Entity {
  commands
    .spawn((
      Name::new("SnakeSegment"),
      SnakeId::new(1),
      SnakeSegment {
        prev: *prev_segment,
      },
      position.clone(),
      Sprite::from_color(
        Color::WHITE,
        Vec2::new(ARENA_CELL_SIZE as f32 / 1.5, ARENA_CELL_SIZE as f32 / 1.5),
      ),
      Transform {
        translation: transform_cell_to_translation(&position),
        ..default()
      },
    ))
    .id()
}

fn spawn_background(commands: &mut Commands, asset_server: Res<AssetServer>) -> (usize, usize) {
  let image = asset_server.load("textures/games/snake/background.png");

  commands.spawn((
    Name::new("SnakeBackground"),
    Transform {
      translation: Vec3::new(0.0, 0.0, -1.0),
      ..Default::default()
    },
    Sprite::from_image(image),
  ));

  // TODO: сделать загрузку из спрайта
  return (IMAGE_WIDTH as usize, IMAGE_HEIGHT as usize);
}

fn accumulate_input_system(
  mut direction_accumulator: ResMut<DirectionAccumulator>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  snake: Single<&SnakeHead>,
) {
  let possible_direction: Vec<SnakeDirection> = vec![
    SnakeDirection::Left,
    SnakeDirection::Right,
    SnakeDirection::Down,
    SnakeDirection::Up,
  ]
  .into_iter()
  .filter(|direction| {
    return *direction != snake.direction.get_opposite();
  })
  .collect();

  if possible_direction.contains(&SnakeDirection::Left)
    && keyboard_input.just_pressed(KeyCode::ArrowLeft)
  {
    direction_accumulator.0 = SnakeDirection::Left;
  } else if possible_direction.contains(&SnakeDirection::Right)
    && keyboard_input.just_pressed(KeyCode::ArrowRight)
  {
    direction_accumulator.0 = SnakeDirection::Right;
  } else if possible_direction.contains(&SnakeDirection::Up)
    && keyboard_input.just_pressed(KeyCode::ArrowUp)
  {
    direction_accumulator.0 = SnakeDirection::Up;
  } else if possible_direction.contains(&SnakeDirection::Down)
    && keyboard_input.just_pressed(KeyCode::ArrowDown)
  {
    direction_accumulator.0 = SnakeDirection::Down;
  }
}

fn move_system(
  mut game_timer: ResMut<GameTimer>,
  mut snake_head_query: Query<(Entity, &mut Transform, &mut GridPosition, &mut SnakeHead)>,
  mut snake_segment_query: Query<
    (Entity, &mut Transform, &mut GridPosition, &mut SnakeSegment),
    Without<SnakeHead>,
  >,
  time: Res<Time>,
  direction_accumulator: Res<DirectionAccumulator>,
) {
  game_timer.tick(time.delta());

  if !game_timer.is_finished() {
    return;
  }

  let mut before_move_positions = snake_segment_query
    .iter()
    .map(|segment| (segment.0, segment.2.clone()))
    .collect::<Vec<(Entity, GridPosition)>>();

  for (entity, _, position, _) in snake_head_query.iter() {
    before_move_positions.push((entity, position.clone()));
  }

  for (_, mut snake_head_transform, mut snake_head_position, mut snake_head) in
    snake_head_query.iter_mut()
  {
    match direction_accumulator.0 {
      SnakeDirection::Left => {
        snake_head.direction = SnakeDirection::Left;
        snake_head_position.decrement_x();
      }
      SnakeDirection::Right => {
        snake_head.direction = SnakeDirection::Right;
        snake_head_position.increment_x();
      }
      SnakeDirection::Down => {
        snake_head.direction = SnakeDirection::Down;
        snake_head_position.decrement_y();
      }
      SnakeDirection::Up => {
        snake_head.direction = SnakeDirection::Up;
        snake_head_position.increment_y();
      }
    }

    snake_head_transform.translation = transform_cell_to_translation(&snake_head_position);

    for (_, mut segment_transform, mut segment_position, segment) in snake_segment_query.iter_mut()
    {
      if let Some(prev_segment) = segment.prev {
        let (_, prev_segment_position) = before_move_positions
          .iter()
          .find(|(entity, _)| entity == &prev_segment)
          .unwrap();

        segment_position.x = prev_segment_position.x;
        segment_position.y = prev_segment_position.y;

        segment_transform.translation = transform_cell_to_translation(&segment_position);
      }
    }
  }
}

fn wait_for_input(
  mut next_state: ResMut<NextState<SnakeGameState>>,
  mut game_timer: ResMut<GameTimer>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  if keyboard_input.just_pressed(KeyCode::Space) {
    game_timer.reset();
    next_state.set(SnakeGameState::Playing);
  }
}

fn transform_cell_to_translation(GridPosition { x, y }: &GridPosition) -> Vec3 {
  const OFFSET_X: f32 = -(ARENA_PIXEL_WIDTH as f32) / 2.0;
  const OFFSET_Y: f32 = -(ARENA_PIXEL_HEIGHT as f32) / 2.0;

  let pixel_x = OFFSET_X + (x * (ARENA_CELL_SIZE + ARENA_CELL_GAP)) as f32;
  let pixel_y = OFFSET_Y + (y * (ARENA_CELL_SIZE + ARENA_CELL_GAP)) as f32;

  Vec3::new(
    pixel_x + (ARENA_CELL_SIZE as f32) / 2.0,
    pixel_y + (ARENA_CELL_SIZE as f32) / 2.0,
    0.0,
  )
}
