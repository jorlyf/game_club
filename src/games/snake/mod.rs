use std::{
  collections::{HashMap, HashSet},
  time::Duration,
};

use bevy::prelude::*;

use crate::{
  game::FontAssets,
  games::{CurrentGameState, GameType},
};

pub struct SnakeGamePlugin;

impl Plugin for SnakeGamePlugin {
  fn build(&self, app: &mut App) {
    app.init_state::<SnakeGameState>();

    app.init_resource::<SnakeGameAssets>();
    app.init_resource::<SnakeSoundAssets>();

    app.init_resource::<DirectionAccumulator>();
    app.init_resource::<GameTimer>();

    app
      .add_systems(Update, setup.run_if(switched_to_game))
      .add_systems(
        PreUpdate,
        (
          wait_for_input_system.run_if(in_state(SnakeGameState::WaitPlayer)),
          wait_for_input_for_restart_system.run_if(in_state(SnakeGameState::GameOver)),
        ),
      )
      .add_systems(
        Update,
        (
          input_accumulation_system.run_if(in_state(SnakeGameState::Playing)),
          snake_movement_system.run_if(in_state(SnakeGameState::Playing)),
          snake_self_collision_system.run_if(in_state(SnakeGameState::Playing)),
          snake_food_collision_system.run_if(in_state(SnakeGameState::Playing)),
          food_spawning_system.run_if(in_state(SnakeGameState::Playing)),
        )
          .chain(),
      )
      .add_observer(start_game)
      .add_observer(food_eaten_observer)
      .add_observer(grow_snake_observer)
      .add_observer(snake_speed_multiplier_reset_observer)
      .add_observer(snake_speed_multiplier_set_observer)
      .add_observer(play_audio_once_observer);

    app
      .add_systems(OnEnter(SnakeGameState::GameOver), game_over_enter_observer)
      .add_systems(OnExit(SnakeGameState::GameOver), game_over_exit_observer);
  }
}

fn switched_to_game(config: Res<CurrentGameState>) -> bool {
  config.is_changed() && config.current_game == Some(GameType::Snake)
}

const ARENA_WIDTH: u32 = 13;
const ARENA_HEIGHT: u32 = 13;
const ARENA_AREA: u32 = ARENA_WIDTH * ARENA_HEIGHT;
const ARENA_CELL_SIZE: u32 = 7;
const ARENA_CELL_GAP: u32 = 1;
const ARENA_PIXEL_WIDTH: u32 = ARENA_WIDTH * ARENA_CELL_SIZE + (ARENA_WIDTH - 1) * ARENA_CELL_GAP;
const ARENA_PIXEL_HEIGHT: u32 =
  ARENA_HEIGHT * ARENA_CELL_SIZE + (ARENA_HEIGHT - 1) * ARENA_CELL_GAP;

const IMAGE_WIDTH: u32 = 252;
const IMAGE_HEIGHT: u32 = 128;

const STEP_TIME_SECONDS: f32 = 0.400;

const START_SNAKE_HEAD_POSITION: GridPosition = GridPosition {
  x: ARENA_WIDTH / 2,
  y: ARENA_HEIGHT / 2,
};
const START_SNAKE_LENGTH: u32 = 2;

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
struct GridPosition {
  x: u32,
  y: u32,
}

impl GridPosition {
  fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }

  fn opposite_to_direction(&self, direction: SnakeDirection) -> Self {
    match direction {
      SnakeDirection::Left => self.right(),
      SnakeDirection::Right => self.left(),
      SnakeDirection::Down => self.up(),
      SnakeDirection::Up => self.down(),
    }
  }

  fn left(&self) -> Self {
    let (mut x, y) = (self.x, self.y);
    if x == 0 {
      x = ARENA_WIDTH - 1;
    } else {
      x -= 1;
    }
    Self::new(x, y)
  }
  fn right(&self) -> Self {
    let (mut x, y) = (self.x, self.y);
    if x == ARENA_WIDTH - 1 {
      x = 0;
    } else {
      x += 1;
    }
    Self::new(x, y)
  }

  fn down(&self) -> Self {
    let (x, mut y) = (self.x, self.y);
    if y == 0 {
      y = ARENA_HEIGHT - 1;
    } else {
      y -= 1;
    }
    Self::new(x, y)
  }
  fn up(&self) -> Self {
    let (x, mut y) = (self.x, self.y);
    if y == ARENA_HEIGHT - 1 {
      y = 0;
    } else {
      y += 1;
    }
    Self::new(x, y)
  }
}

#[derive(Component)]
struct SnakeHead {
  direction: SnakeDirection,
}

#[derive(Component, Debug)]
struct SnakeSegment {
  follow_to: Option<Entity>,
}

impl Default for SnakeHead {
  fn default() -> Self {
    Self {
      direction: SnakeDirection::Left,
    }
  }
}

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq)]
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

#[derive(Component, Clone)]
enum Food {
  Green {
    growth_amount: u32,
  },
  Red {
    growth_amount: u32,
    speed_multiplier: f32,
  },
  Blue {
    speed_multiplier: f32,
  },
}

#[derive(Event)]
struct FoodEatenEvent {
  food_entity: Entity,
}

#[derive(Event)]
struct SnakeGrowEvent {
  amount: u32,
}

#[derive(Event)]
struct RequestStartGameEvent;

#[derive(Event)]
struct SnakeSpeedMultiplierResetEvent;

#[derive(Event)]
struct SnakeSpeedMultiplierSetEvent {
  multiplier: f32,
}

#[derive(Event)]
struct PlayAudioOnceEvent {
  sound_handle: Handle<AudioSource>,
}

#[derive(States, Debug, Clone, Hash, Eq, PartialEq, Default)]
enum SnakeGameState {
  #[default]
  NotStarted,
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

impl GameTimer {
  fn reset_duration(&mut self) {
    self
      .0
      .set_duration(Duration::from_secs_f32(STEP_TIME_SECONDS));
  }

  fn set_duration_multiplier(&mut self, multiplier: f32) {
    self
      .0
      .set_duration(Duration::from_secs_f32(
        self.0.duration().as_secs_f32() * 1.0 / multiplier,
      ));
  }
}

impl Default for GameTimer {
  fn default() -> Self {
    GameTimer(Timer::from_seconds(STEP_TIME_SECONDS, TimerMode::Repeating))
  }
}

#[derive(Component)]
struct GameOverUi;

#[derive(Component)]
struct MenuUi;

#[derive(Resource, Default)]
struct SnakeGameAssets {
  game_over: Handle<Image>,
  background: Handle<Image>,
}

#[derive(Resource, Default)]
struct SnakeSoundAssets {
  eat_green: Handle<AudioSource>,
  eat_red: Handle<AudioSource>,
  eat_blue: Handle<AudioSource>,
}

fn setup(
  mut commands: Commands,
  mut next_state: ResMut<NextState<SnakeGameState>>,
  mut snake_game_assets: ResMut<SnakeGameAssets>,
  mut sound_assets: ResMut<SnakeSoundAssets>,
  asset_server: Res<AssetServer>,
) {
  snake_game_assets.game_over = asset_server.load("games/snake/game_over.png");
  snake_game_assets.background = asset_server.load("games/snake/background.png");

  sound_assets.eat_green = asset_server.load("games/snake/sounds/green_food_pickup.wav");
  sound_assets.eat_red = asset_server.load("games/snake/sounds/red_food_pickup.wav");
  sound_assets.eat_blue = asset_server.load("games/snake/sounds/blue_food_pickup.wav");

  let (width, height) = spawn_background(&mut commands, snake_game_assets.background.clone());

  let mut projection = OrthographicProjection::default_2d();
  projection.scaling_mode = bevy::camera::ScalingMode::Fixed {
    width: width as f32,
    height: height as f32,
  };

  spawn_camera(&mut commands, &projection);

  commands.trigger(RequestStartGameEvent);

  next_state.set(SnakeGameState::WaitPlayer);
}

fn start_game(
  _: On<RequestStartGameEvent>,
  mut commands: Commands,
  direction_accumulator: Res<DirectionAccumulator>,
) {
  let snake_head_direction = direction_accumulator.0;

  let snake_head_entity = spawn_snake_head_segment(
    &mut commands,
    &START_SNAKE_HEAD_POSITION,
    snake_head_direction,
  );

  assert!(START_SNAKE_LENGTH > 0);

  let mut last_segment = (
    snake_head_entity,
    START_SNAKE_HEAD_POSITION.opposite_to_direction(snake_head_direction),
  );
  for _ in 1..START_SNAKE_LENGTH {
    let snake_segment_entity =
      spawn_snake_body_segment(&mut commands, (last_segment.0, &last_segment.1));

    last_segment = (
      snake_segment_entity,
      last_segment
        .1
        .opposite_to_direction(snake_head_direction),
    );
  }
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

fn spawn_snake_head_segment(
  commands: &mut Commands,
  position: &GridPosition,
  direction: SnakeDirection,
) -> Entity {
  commands
    .spawn((
      Name::new("SnakeHead"),
      SnakeHead { direction },
      SnakeSegment { follow_to: None },
      GridPosition::clone(position),
      Transform {
        translation: transform_cell_to_translation(&position),
        ..Default::default()
      },
      Sprite::from_color(
        Color::srgb(1.0, 1.0, 0.0),
        Vec2::new(ARENA_CELL_SIZE as f32 / 1.5, ARENA_CELL_SIZE as f32 / 1.5),
      ),
    ))
    .id()
}

fn spawn_snake_body_segment(commands: &mut Commands, follow_to: (Entity, &GridPosition)) -> Entity {
  let (follow_to_entity, follow_to_position) = follow_to;

  commands
    .spawn((
      Name::new("SnakeBody"),
      SnakeSegment {
        follow_to: Some(follow_to_entity),
      },
      GridPosition::clone(follow_to_position),
      Transform {
        translation: transform_cell_to_translation(&follow_to_position),
        ..Default::default()
      },
      Sprite::from_color(
        Color::srgb(1.0, 1.0, 1.0),
        Vec2::new(ARENA_CELL_SIZE as f32 / 1.5, ARENA_CELL_SIZE as f32 / 1.5),
      ),
    ))
    .id()
}

fn grow_snake_observer(
  grow_event: On<SnakeGrowEvent>,
  mut commands: Commands,
  snake_segment_query: Query<(Entity, &GridPosition, &SnakeSegment)>,
) {
  let segments = snake_segment_query
    .iter()
    .map(|(entity, _, segment)| (entity, segment))
    .collect::<Vec<_>>();

  let mut follow_to_entity = find_snake_tail_segment(&segments);

  let follow_to_position = snake_segment_query
    .get(follow_to_entity)
    .unwrap()
    .1;

  for _ in 0..grow_event.amount {
    follow_to_entity = commands
      .spawn((
        Name::new("SnakeBody"),
        SnakeSegment {
          follow_to: Some(follow_to_entity),
        },
        GridPosition::clone(follow_to_position),
        Transform {
          translation: transform_cell_to_translation(&follow_to_position),
          ..Default::default()
        },
        Sprite::from_color(
          Color::srgb(1.0, 1.0, 1.0),
          Vec2::new(ARENA_CELL_SIZE as f32 / 1.5, ARENA_CELL_SIZE as f32 / 1.5),
        ),
      ))
      .id();
  }
}

fn snake_speed_multiplier_reset_observer(
  _: On<SnakeSpeedMultiplierResetEvent>,
  mut game_timer: ResMut<GameTimer>,
) {
  game_timer.reset_duration();
}

fn snake_speed_multiplier_set_observer(
  speed_multiplier_event: On<SnakeSpeedMultiplierSetEvent>,
  mut game_timer: ResMut<GameTimer>,
) {
  game_timer.set_duration_multiplier(speed_multiplier_event.multiplier);
}

fn food_eaten_observer(
  eaten_food: On<FoodEatenEvent>,
  mut commands: Commands,
  food_query: Query<(Entity, &GridPosition, &Food)>,
  sound_assets: Res<SnakeSoundAssets>,
) {
  let eaten_food = food_query
    .get(eaten_food.food_entity)
    .unwrap();

  match *eaten_food.2 {
    Food::Green { growth_amount } => {
      commands.trigger(SnakeGrowEvent {
        amount: growth_amount,
      });
      commands.trigger(PlayAudioOnceEvent {
        sound_handle: sound_assets.eat_green.clone(),
      });
    }
    Food::Red {
      growth_amount,
      speed_multiplier,
    } => {
      commands.trigger(SnakeGrowEvent {
        amount: growth_amount,
      });
      commands.trigger(SnakeSpeedMultiplierSetEvent {
        multiplier: speed_multiplier,
      });
      commands.trigger(PlayAudioOnceEvent {
        sound_handle: sound_assets.eat_red.clone(),
      });
    }
    Food::Blue { speed_multiplier } => {
      commands.trigger(SnakeSpeedMultiplierSetEvent {
        multiplier: speed_multiplier,
      });
      commands.trigger(PlayAudioOnceEvent {
        sound_handle: sound_assets.eat_blue.clone(),
      });
    }
  }

  commands.entity(eaten_food.0).despawn();
}

fn play_audio_once_observer(event: On<PlayAudioOnceEvent>, mut commands: Commands) {
  commands.spawn((
    Name::new("AudioSource"),
    AudioPlayer::new(event.sound_handle.clone()),
  ));
}

fn spawn_background(commands: &mut Commands, background_image: Handle<Image>) -> (usize, usize) {
  commands.spawn((
    Name::new("Background"),
    Transform {
      translation: Vec3::new(0.0, 0.0, -1.0),
      ..Default::default()
    },
    Sprite::from_image(background_image),
  ));

  // TODO: сделать загрузку из спрайта
  return (IMAGE_WIDTH as usize, IMAGE_HEIGHT as usize);
}

fn spawn_food(commands: &mut Commands, food: Food, position: GridPosition) -> Entity {
  // let image = asset_server.load("games/snake/food.png");

  let color = match &food {
    Food::Green { .. } => Color::srgb(0.0, 1.0, 0.0),
    Food::Red { .. } => Color::srgb(1.0, 0.0, 0.0),
    Food::Blue { .. } => Color::srgb(0.0, 0.0, 1.0),
  };

  commands
    .spawn((
      Name::new("Food"),
      food,
      position.clone(),
      Sprite::from_color(
        color,
        Vec2::new(ARENA_CELL_SIZE as f32 / 1.5, ARENA_CELL_SIZE as f32 / 1.5),
      ),
      Transform {
        translation: transform_cell_to_translation(&position),
        ..Default::default()
      },
    ))
    .id()
}

fn input_accumulation_system(
  mut direction_accumulator: ResMut<DirectionAccumulator>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  snake_head: Single<&SnakeHead>,
) {
  let possible_direction: Vec<SnakeDirection> = vec![
    SnakeDirection::Left,
    SnakeDirection::Right,
    SnakeDirection::Down,
    SnakeDirection::Up,
  ]
  .into_iter()
  .filter(|direction| {
    return *direction != snake_head.direction.get_opposite();
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

fn snake_movement_system(
  mut game_timer: ResMut<GameTimer>,
  mut snake_segment_query: Query<
    (Entity, &mut Transform, &mut GridPosition, &mut SnakeSegment),
    Without<SnakeHead>,
  >,
  snake_head_single: Single<(Entity, &mut Transform, &mut GridPosition, &mut SnakeHead)>,
  time: Res<Time>,
  direction_accumulator: Res<DirectionAccumulator>,
) {
  game_timer.tick(time.delta());

  if !game_timer.is_finished() {
    return;
  }

  let (snake_head_entity, mut snake_head_transform, mut snake_head_position, mut snake_head) =
    snake_head_single.into_inner();

  let mut before_move_segment_positions = snake_segment_query
    .iter()
    .map(|segment| (segment.0, segment.2.clone()))
    .collect::<Vec<(Entity, GridPosition)>>();

  before_move_segment_positions.insert(0, (snake_head_entity, snake_head_position.clone()));

  match direction_accumulator.0 {
    SnakeDirection::Left => {
      snake_head.direction = SnakeDirection::Left;
      *snake_head_position = snake_head_position.left();
    }
    SnakeDirection::Right => {
      snake_head.direction = SnakeDirection::Right;
      *snake_head_position = snake_head_position.right();
    }
    SnakeDirection::Down => {
      snake_head.direction = SnakeDirection::Down;
      *snake_head_position = snake_head_position.down();
    }
    SnakeDirection::Up => {
      snake_head.direction = SnakeDirection::Up;
      *snake_head_position = snake_head_position.up();
    }
  }

  snake_head_transform.translation = transform_cell_to_translation(&snake_head_position);

  for (_, mut segment_transform, mut segment_position, segment) in snake_segment_query.iter_mut() {
    if let Some(follow_to_segment_entity) = segment.follow_to {
      let (_, follow_to_segment_position) = before_move_segment_positions
        .iter()
        .find(|(entity, _)| entity == &follow_to_segment_entity)
        .unwrap();

      *segment_position = GridPosition::clone(follow_to_segment_position);

      segment_transform.translation = transform_cell_to_translation(&segment_position);
    }
  }
}

fn food_spawning_system(
  mut commands: Commands,
  snake_segment_position_query: Query<&GridPosition, With<SnakeSegment>>,
  food_query: Query<(&GridPosition, &Food)>,
) {
  let mut except: Vec<GridPosition> = snake_segment_position_query
    .iter()
    .map(|p| p.clone())
    .chain(
      food_query
        .iter()
        .map(|(p, _)| p.clone()),
    )
    .collect();

  fn has_food_type(food_query: &Query<(&GridPosition, &Food)>, target: &Food) -> bool {
    food_query
      .iter()
      .any(|(_, food)| std::mem::discriminant(food) == std::mem::discriminant(target))
  }

  let spawn_if_missing = |food: Food, except: &mut Vec<GridPosition>, commands: &mut Commands| {
    if !has_food_type(&food_query, &food) {
      let position = get_random_position_except(except);
      except.push(position.clone());
      spawn_food(commands, food, position);
    }
  };

  let green = Food::Green { growth_amount: 1 };
  let red = Food::Red {
    growth_amount: 3,
    speed_multiplier: 1.25,
  };
  let blue = Food::Blue {
    speed_multiplier: 0.85,
  };

  spawn_if_missing(green, &mut except, &mut commands);
  spawn_if_missing(red, &mut except, &mut commands);
  spawn_if_missing(blue, &mut except, &mut commands);
}

fn snake_self_collision_system(
  mut next_state: ResMut<NextState<SnakeGameState>>,
  head_single: Single<(Entity, &GridPosition), With<SnakeHead>>,
  segment_query: Query<(Entity, &GridPosition), With<SnakeSegment>>,
) {
  let (head_entity, head_position) = head_single.into_inner();

  for (segment_entity, segment_position) in segment_query.iter() {
    if head_entity == segment_entity {
      continue;
    }

    if *head_position == *segment_position {
      next_state.set(SnakeGameState::GameOver);
      return;
    }
  }
}

fn snake_food_collision_system(
  mut commands: Commands,
  segment_query: Query<&GridPosition, With<SnakeSegment>>,
  food_query: Query<(Entity, &GridPosition, &Food)>,
) {
  let mut foods = HashMap::new();

  for (entity, position, food) in food_query.iter() {
    foods.insert(position, (entity, food));
  }

  for position in segment_query.iter() {
    if foods.contains_key(&position) {
      commands.trigger(FoodEatenEvent {
        food_entity: foods.get(&position).unwrap().0,
      });
      return;
    }
  }
}

fn game_over_enter_observer(
  mut commands: Commands,
  font_assets: Res<FontAssets>,
  snake_game_assets: Res<SnakeGameAssets>,
) {
  let create_game_over_ui = |game_over_image: Handle<Image>| {
    let image_node = (
      Node {
        height: px(300.),
        ..default()
      },
      children![
        (ImageNode {
          image: game_over_image,
          ..default()
        })
      ],
    );

    let text_node = (
      Text {
        0: String::from("Press any key to restart..."),
        ..Default::default()
      },
      TextFont {
        font: font_assets.regular.clone(),
        font_size: 32.,
        ..Default::default()
      },
      TextColor(Color::WHITE),
    );

    (
      Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
      },
      children![image_node, text_node],
    )
  };

  commands.spawn((
    create_game_over_ui(snake_game_assets.game_over.clone()),
    GameOverUi,
  ));
}

fn game_over_exit_observer(
  mut commands: Commands,
  game_over_ui_query: Query<Entity, With<GameOverUi>>,
) {
  for entity in game_over_ui_query.iter() {
    commands.entity(entity).despawn();
  }
}

fn wait_for_input_system(
  mut direction_accumulator: ResMut<DirectionAccumulator>,
  mut next_state: ResMut<NextState<SnakeGameState>>,
  mut game_timer: ResMut<GameTimer>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  const INPUTS: [KeyCode; 4] = [
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
  ];

  let mut head_direction = SnakeDirection::Left;

  if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
    head_direction = SnakeDirection::Left;
  } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
    head_direction = SnakeDirection::Right;
  } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
    head_direction = SnakeDirection::Up;
  } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
    head_direction = SnakeDirection::Down;
  }

  direction_accumulator.0 = head_direction;

  if keyboard_input.any_just_pressed(INPUTS) {
    game_timer.reset_duration();
    game_timer.reset();
    next_state.set(SnakeGameState::Playing);
  }
}

fn wait_for_input_for_restart_system(
  mut commands: Commands,
  mut direction_accumulator: ResMut<DirectionAccumulator>,
  mut next_state: ResMut<NextState<SnakeGameState>>,
  mut game_timer: ResMut<GameTimer>,
  mut snake_segment_query: Query<Entity, With<SnakeSegment>>,
  mut food_query: Query<Entity, With<Food>>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  const INPUTS: [KeyCode; 4] = [
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
  ];

  let mut head_direction = SnakeDirection::Left;

  if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
    head_direction = SnakeDirection::Left;
  } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
    head_direction = SnakeDirection::Right;
  } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
    head_direction = SnakeDirection::Up;
  } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
    head_direction = SnakeDirection::Down;
  }

  direction_accumulator.0 = head_direction;

  if keyboard_input.any_just_pressed(INPUTS) {
    snake_segment_query
      .iter_mut()
      .for_each(|food| {
        commands.entity(food).despawn();
      });

    food_query.iter_mut().for_each(|food| {
      commands.entity(food).despawn();
    });

    commands.trigger(RequestStartGameEvent);

    game_timer.reset_duration();
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

fn get_random_position_except(except: &Vec<GridPosition>) -> GridPosition {
  let except = except.iter().collect::<HashSet<_>>();

  let mut i = rand::random_range(0..ARENA_AREA as usize - except.len());

  for y in 0..ARENA_HEIGHT {
    for x in 0..ARENA_WIDTH {
      if except.contains(&GridPosition { x, y }) {
        continue;
      }
      if i == 0 {
        return GridPosition { x, y };
      }
      i -= 1;
    }
  }

  panic!("Failed to find a random position");
}

fn find_snake_tail_segment(segments: &[(Entity, &SnakeSegment)]) -> Entity {
  let mut referenced = HashSet::<Entity>::new();

  for (_, segment) in segments.iter() {
    if let Some(follow_to) = segment.follow_to {
      referenced.insert(follow_to);
    }
  }

  for (entity, _) in segments.iter() {
    if !referenced.contains(entity) {
      return *entity;
    }
  }

  panic!("No tail segment found");
}
